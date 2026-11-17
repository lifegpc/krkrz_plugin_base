use super::{EXECUTOR, RUNTIME};
use anyhow::{Context, Result, anyhow, bail};
use bytes::Bytes;
use krkrz_plugin_base::{com::IStreamWrapper, tp_stub::*, *};
use reqwest::header::{HeaderName, HeaderValue};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::ptr;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use windows::Win32::Globalization::{
    CMultiLanguage, IMultiLanguage, MIMECSETINFO, MultiByteToWideChar, WideCharToMultiByte,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};

const STORAGE_READ: u32 = 0;
const STORAGE_WRITE: u32 = 1;

static mut MULTILANGUAGE: Option<IMultiLanguage> = None;
static mut COM_INITIALIZED: bool = false;

pub(super) fn initialize_encoding() {
    let initialized = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok() };
    if !initialized {
        return;
    }
    unsafe {
        COM_INITIALIZED = true;
        MULTILANGUAGE = CoCreateInstance(&CMultiLanguage, None, CLSCTX_ALL).ok();
    }
}

pub(super) fn uninitialize_encoding() {
    unsafe {
        MULTILANGUAGE = None;
        if COM_INITIALIZED {
            CoUninitialize();
            COM_INITIALIZED = false;
        }
    }
}

#[derive(Clone)]
struct RequestSpec {
    client: reqwest::Client,
    method: reqwest::Method,
    url: reqwest::Url,
    headers: Vec<(HeaderName, HeaderValue)>,
    encoding: Option<String>,
    auth: Option<(String, String)>,
}

struct RequestState {
    ready_state: i64,
    status: i64,
    status_text: String,
    content_type: String,
    content_type_encoding: String,
    content_length: u64,
    response_headers: BTreeMap<String, String>,
    response_data: Option<Vec<u8>>,
    request: Option<RequestSpec>,
    running: bool,
    generation: u64,
    operation_id: u64,
    cancellation: Option<CancellationToken>,
    pending_open_event: Option<oneshot::Receiver<()>>,
}

impl Default for RequestState {
    fn default() -> Self {
        Self {
            ready_state: 0,
            status: 0,
            status_text: String::new(),
            content_type: String::new(),
            content_type_encoding: String::new(),
            content_length: 0,
            response_headers: BTreeMap::new(),
            response_data: None,
            request: None,
            running: false,
            generation: 0,
            operation_id: 0,
            cancellation: None,
            pending_open_event: None,
        }
    }
}

struct StorageReader(IStreamWrapper);

// The reference plugin reads TVP streams from its worker thread. The wrapper owns
// the COM reference and is only accessed by one blocking worker at a time.
unsafe impl Send for StorageReader {}

struct StorageWriter(IStreamWrapper);

// The stream is opened on the window thread and then exclusively owned by the
// request task until it is flushed and dropped.
unsafe impl Send for StorageWriter {}

enum RequestBody {
    Empty,
    Bytes(Vec<u8>),
    Storage(StorageReader, u64),
}

impl RequestBody {
    fn content_length(&self) -> u64 {
        match self {
            Self::Empty => 0,
            Self::Bytes(data) => data.len() as u64,
            Self::Storage(_, length) => *length,
        }
    }
}

#[derive(Clone, Copy)]
enum HttpEvent {
    ReadyState(i64),
    Progress(bool, f64),
}

struct HttpRequestTjsObj(*mut iTJSDispatch2);

impl HttpRequestTjsObj {
    fn new(obj: *mut iTJSDispatch2) -> Self {
        if !obj.is_null() {
            unsafe { (*obj).add_ref() };
        }
        Self(obj)
    }

    fn invoke<const N: usize>(&self, name: *const tjs_char, args: &mut Tjs2ArgsArray<N>) {
        if self.0.is_null() {
            return;
        }
        let object = unsafe { &mut *self.0 };
        let mut value = tTJSVariant::new();
        let result = unsafe { object.prop_get(0, name, ptr::null_mut(), &mut value, self.0) };
        if TJS_FAILED(result) || !value.is_object() {
            return;
        }
        let closure = value.as_object_closure();
        let _ = unsafe {
            closure.func_call(
                0,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                args.len() as tjs_int,
                args.as_mut_ptr(),
                self.0,
            )
        };
    }

    fn invoke_event(&self, event: HttpEvent) {
        match event {
            HttpEvent::ReadyState(ready_state) => {
                let mut args = tjs2_get_args!(ready_state);
                self.invoke(tjs_w!("onReadyStateChange"), &mut args);
            }
            HttpEvent::Progress(upload, percent) => {
                let mut args = tjs2_get_args!(upload, percent);
                self.invoke(tjs_w!("onProgress"), &mut args);
            }
        }
    }
}

impl Drop for HttpRequestTjsObj {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { (*self.0).release() };
        }
    }
}

impl Clone for HttpRequestTjsObj {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

unsafe impl Send for HttpRequestTjsObj {}

async fn dispatch_event(
    executor: AsyncExecutor,
    object: HttpRequestTjsObj,
    state: Arc<Mutex<RequestState>>,
    generation: u64,
    event: HttpEvent,
) {
    executor
        .execute(move || {
            let should_invoke = {
                let mut state = state.lock().unwrap();
                if state.generation != generation {
                    false
                } else {
                    if let HttpEvent::ReadyState(ready_state) = event {
                        state.ready_state = ready_state;
                    }
                    true
                }
            };
            if should_invoke {
                object.invoke_event(event);
            }
        })
        .await;
}

fn schedule_event(
    this: *mut iTJSDispatch2,
    state: Arc<Mutex<RequestState>>,
    generation: u64,
    event: HttpEvent,
    wait_for_completion: bool,
) -> Result<Option<oneshot::Receiver<()>>> {
    let (runtime, executor) = callback_context()?;
    let object = HttpRequestTjsObj::new(this);
    let (sender, receiver) = if wait_for_completion {
        let (sender, receiver) = oneshot::channel();
        (Some(sender), Some(receiver))
    } else {
        (None, None)
    };
    runtime.spawn(async move {
        dispatch_event(executor, object, state, generation, event).await;
        if let Some(sender) = sender {
            let _ = sender.send(());
        }
    });
    Ok(receiver)
}

fn callback_context() -> Result<(tokio::runtime::Handle, AsyncExecutor)> {
    unsafe {
        let runtime = (&raw const RUNTIME)
            .as_ref()
            .and_then(|runtime| runtime.as_ref())
            .ok_or_else(|| anyhow!("tokio runtime is not initialized"))?;
        let executor = (&raw const EXECUTOR)
            .as_ref()
            .and_then(|executor| executor.as_ref())
            .cloned()
            .ok_or_else(|| anyhow!("AsyncExecutor is not initialized"))?;
        Ok((runtime.handle().clone(), executor))
    }
}

fn parse_content_type(value: &str) -> (String, String) {
    let mut parts = value.split(';');
    let media_type = parts.next().unwrap_or_default().trim().to_owned();
    let encoding = parts
        .find_map(|part| {
            let (name, value) = part.split_once('=')?;
            name.trim()
                .eq_ignore_ascii_case("charset")
                .then(|| value.trim().trim_matches(['\'', '"']).to_owned())
        })
        .unwrap_or_default();
    (media_type, encoding)
}

fn html_meta_content_type(first_chunk: &[u8]) -> Option<String> {
    let html = String::from_utf8_lossy(first_chunk);
    let lower = html.to_ascii_lowercase();
    let mut offset = 0;
    while let Some(relative_start) = lower[offset..].find("<meta") {
        let start = offset + relative_start;
        let end = lower[start..].find('>')? + start;
        let tag = &html[start..=end];
        let lower_tag = &lower[start..=end];
        if lower_tag.contains("http-equiv") && lower_tag.contains("content-type") {
            if let Some(content_start) = lower_tag.find("content") {
                if let Some(equal_offset) = lower_tag[content_start..].find('=') {
                    let value_start = content_start + equal_offset + 1;
                    let remainder = tag[value_start..].trim_start();
                    let value = if let Some(quote) =
                        remainder.chars().next().filter(|c| *c == '\'' || *c == '"')
                    {
                        let rest = &remainder[quote.len_utf8()..];
                        rest.find(quote).map(|end| &rest[..end])?
                    } else {
                        remainder
                            .split(|c: char| c.is_ascii_whitespace() || c == '>')
                            .next()
                            .unwrap_or_default()
                    };
                    if !value.is_empty() {
                        return Some(value.to_owned());
                    }
                }
            }
        }
        offset = end + 1;
    }
    None
}

fn windows_code_page(label: Option<&str>) -> u32 {
    if let Some(label) = label {
        let charset = windows::core::BSTR::from(label);
        let mut info = MIMECSETINFO::default();
        let found = unsafe {
            (&raw const MULTILANGUAGE)
                .as_ref()
                .and_then(|multi| multi.as_ref())
                .is_some_and(|multi| multi.GetCharsetInfo(&charset, &mut info).is_ok())
        };
        if found {
            return info.uiInternetEncoding;
        }
    }
    let normalized = label
        .unwrap_or("utf-8")
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '_', ' '], "");
    match normalized.as_str() {
        "utf8" | "unicode11utf8" => 65001,
        "utf16" | "utf16le" | "unicode" => 1200,
        "utf16be" | "unicodefffe" => 1201,
        "sjis" | "shiftjis" | "mskanji" | "windows31j" | "cp932" => 932,
        "eucjp" => 51932,
        "iso2022jp" => 50220,
        "gbk" | "cp936" | "gb2312" => 936,
        "gb18030" => 54936,
        "big5" | "cp950" => 950,
        "euckr" | "cp949" => 949,
        "ascii" | "usascii" => 20127,
        "latin1" | "iso88591" | "windows1252" | "cp1252" => 1252,
        "iso88592" | "windows1250" | "cp1250" => 1250,
        "windows1251" | "cp1251" => 1251,
        "windows1253" | "cp1253" => 1253,
        "windows1254" | "cp1254" => 1254,
        "windows1255" | "cp1255" => 1255,
        "windows1256" | "cp1256" => 1256,
        "windows1257" | "cp1257" => 1257,
        "windows1258" | "cp1258" => 1258,
        "iso885915" => 28605,
        "koi8r" => 20866,
        "koi8u" => 21866,
        _ => 65001,
    }
}

fn decode_text(bytes: &[u8], encoding: Option<&str>) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let code_page = windows_code_page(encoding);
    let length = unsafe { MultiByteToWideChar(code_page, Default::default(), bytes, None) };
    if length <= 0 {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let mut wide = vec![0u16; length as usize];
    let converted =
        unsafe { MultiByteToWideChar(code_page, Default::default(), bytes, Some(&mut wide)) };
    if converted <= 0 {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    String::from_utf16_lossy(&wide[..converted as usize])
}

fn encode_text(text: &str, encoding: Option<&str>) -> Vec<u8> {
    if text.is_empty() {
        return Vec::new();
    }
    let wide: Vec<u16> = text.encode_utf16().collect();
    let code_page = windows_code_page(encoding);
    let length = unsafe {
        WideCharToMultiByte(
            code_page,
            0,
            &wide,
            None,
            windows::core::PCSTR::null(),
            None,
        )
    };
    if length <= 0 {
        return text.as_bytes().to_vec();
    }
    let mut bytes = vec![0u8; length as usize];
    let converted = unsafe {
        WideCharToMultiByte(
            code_page,
            0,
            &wide,
            Some(&mut bytes),
            windows::core::PCSTR::null(),
            None,
        )
    };
    if converted <= 0 {
        text.as_bytes().to_vec()
    } else {
        bytes.truncate(converted as usize);
        bytes
    }
}

fn octet_variant(bytes: &[u8]) -> tTJSVariant {
    if bytes.is_empty() {
        tTJSVariant::new()
    } else {
        tTJSVariant::from((bytes.as_ptr(), bytes.len() as tjs_uint))
    }
}

fn body_from_variant(data: Option<&mut tTJSVariant>, encoding: Option<&str>) -> RequestBody {
    let Some(data) = data else {
        return RequestBody::Empty;
    };
    if data.is_string() {
        let Ok(text) = String::to_param(data) else {
            return RequestBody::Empty;
        };
        RequestBody::Bytes(encode_text(&text, encoding))
    } else if data.is_octet() {
        match types::Octet::to_param(data) {
            Ok(bytes) => RequestBody::Bytes(bytes.to_vec()),
            Err(_) => RequestBody::Empty,
        }
    } else {
        RequestBody::Empty
    }
}

fn open_storage_reader(name: &str) -> Result<(StorageReader, u64)> {
    let stream =
        create_istream(name, STORAGE_READ).ok_or_else(|| anyhow!("sendStorage open failed"))?;
    let mut stream = IStreamWrapper(stream);
    let length = stream
        .stream_length()
        .map_err(|_| anyhow!("sendStorage open failed"))?;
    Ok((StorageReader(stream), length))
}

fn open_storage_writer(name: &str) -> Result<StorageWriter> {
    create_istream(name, STORAGE_WRITE)
        .map(|stream| StorageWriter(IStreamWrapper(stream)))
        .ok_or_else(|| anyhow!("saveStorage open failed"))
}

fn begin_request(
    state: &Arc<Mutex<RequestState>>,
    body: &RequestBody,
    output: Option<StorageWriter>,
    asynchronous: bool,
) -> Result<(
    RequestSpec,
    CancellationToken,
    u64,
    u64,
    Option<StorageWriter>,
    Option<oneshot::Receiver<()>>,
)> {
    let mut state = state.lock().unwrap();
    if state.running {
        bail!("already running");
    }
    let mut request = state
        .request
        .as_ref()
        .ok_or_else(|| anyhow!("not open"))?
        .clone();
    let content_length = body.content_length();
    if content_length > 0
        && !request
            .headers
            .iter()
            .any(|(name, _)| name == reqwest::header::CONTENT_LENGTH)
    {
        request.headers.push((
            reqwest::header::CONTENT_LENGTH,
            HeaderValue::from_str(&content_length.to_string())?,
        ));
    }

    let generation = state.generation;
    state.operation_id += 1;
    let operation_id = state.operation_id;
    let cancellation = CancellationToken::new();
    state.cancellation = Some(cancellation.clone());
    state.running = true;
    state.status = 0;
    state.status_text.clear();
    state.content_type.clear();
    state.content_type_encoding.clear();
    state.content_length = 0;
    state.response_headers.clear();
    state.response_data = None;
    let pending_open_event = if asynchronous {
        state.pending_open_event.take()
    } else {
        None
    };
    Ok((
        request,
        cancellation,
        generation,
        operation_id,
        output,
        pending_open_event,
    ))
}

async fn request_body(
    body: RequestBody,
    state: Arc<Mutex<RequestState>>,
    generation: u64,
    callbacks: Option<(AsyncExecutor, HttpRequestTjsObj)>,
) -> Option<reqwest::Body> {
    match body {
        RequestBody::Empty => None,
        RequestBody::Bytes(bytes) => Some(reqwest::Body::from(bytes)),
        RequestBody::Storage(reader, length) => {
            let (sender, receiver) = mpsc::channel::<std::io::Result<Bytes>>(2);
            tokio::task::spawn_blocking(move || {
                let mut reader = reader;
                let mut buffer = vec![0u8; 64 * 1024];
                loop {
                    match reader.0.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(read) => {
                            if sender
                                .blocking_send(Ok(Bytes::copy_from_slice(&buffer[..read])))
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(error) => {
                            let _ = sender.blocking_send(Err(error));
                            break;
                        }
                    }
                }
            });

            let stream = futures_util::stream::unfold(
                (receiver, 0u64, state, callbacks),
                move |(mut receiver, mut loaded, state, callbacks)| async move {
                    let item = receiver.recv().await?;
                    if let Ok(bytes) = &item {
                        loaded += bytes.len() as u64;
                        if let Some((executor, object)) = callbacks.as_ref() {
                            let percent = if length == 0 {
                                100.0
                            } else {
                                (loaded as f64 * 100.0 / length as f64).min(100.0)
                            };
                            dispatch_event(
                                executor.clone(),
                                object.clone(),
                                Arc::clone(&state),
                                generation,
                                HttpEvent::Progress(true, percent),
                            )
                            .await;
                        }
                    }
                    Some((item, (receiver, loaded, state, callbacks)))
                },
            );
            Some(reqwest::Body::wrap_stream(stream))
        }
    }
}

struct RunningGuard {
    state: Arc<Mutex<RequestState>>,
    generation: u64,
    operation_id: u64,
    asynchronous: bool,
}

impl Drop for RunningGuard {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        if state.generation == self.generation
            && state.operation_id == self.operation_id
            && state.running
        {
            state.running = false;
            state.cancellation = None;
            if self.asynchronous {
                state.ready_state = 4;
            }
        }
    }
}

async fn run_request(
    state: Arc<Mutex<RequestState>>,
    request: RequestSpec,
    body: RequestBody,
    mut output: Option<StorageWriter>,
    cancellation: CancellationToken,
    pending_open_event: Option<oneshot::Receiver<()>>,
    generation: u64,
    operation_id: u64,
    callbacks: Option<(AsyncExecutor, HttpRequestTjsObj)>,
) -> i64 {
    let _running_guard = RunningGuard {
        state: Arc::clone(&state),
        generation,
        operation_id,
        asynchronous: callbacks.is_some(),
    };
    if let Some(pending_open_event) = pending_open_event {
        let _ = pending_open_event.await;
    }
    if let Some((executor, object)) = callbacks.as_ref() {
        dispatch_event(
            executor.clone(),
            object.clone(),
            Arc::clone(&state),
            generation,
            HttpEvent::ReadyState(2),
        )
        .await;
    }

    let mut builder = request
        .client
        .request(request.method.clone(), request.url.clone());
    for (name, value) in &request.headers {
        builder = builder.header(name, value);
    }
    if let Some((username, password)) = &request.auth {
        builder = builder.basic_auth(username, Some(password));
    }
    let has_upload = body.content_length() > 0;
    let upload_callbacks = callbacks
        .as_ref()
        .map(|(executor, object)| (executor.clone(), object.clone()));
    if let Some(body) = request_body(body, Arc::clone(&state), generation, upload_callbacks).await {
        builder = builder.body(body);
    }

    let response = tokio::select! {
        _ = cancellation.cancelled() => None,
        result = builder.send() => Some(result),
    };
    let mut response = match response {
        None => {
            finish_request(
                &state,
                generation,
                operation_id,
                -1,
                "aborted",
                None,
                None,
                0,
                callbacks.is_some(),
            );
            if let Some((executor, object)) = callbacks {
                dispatch_event(
                    executor,
                    object,
                    Arc::clone(&state),
                    generation,
                    HttpEvent::ReadyState(4),
                )
                .await;
            }
            return -1;
        }
        Some(Err(error)) => {
            let message = error.to_string();
            finish_request(
                &state,
                generation,
                operation_id,
                0,
                &message,
                None,
                None,
                0,
                callbacks.is_some(),
            );
            if let Some((executor, object)) = callbacks {
                dispatch_event(
                    executor,
                    object,
                    Arc::clone(&state),
                    generation,
                    HttpEvent::ReadyState(4),
                )
                .await;
            }
            return 0;
        }
        Some(Ok(response)) => response,
    };

    if has_upload {
        if let Some((executor, object)) = callbacks.as_ref() {
            dispatch_event(
                executor.clone(),
                object.clone(),
                Arc::clone(&state),
                generation,
                HttpEvent::Progress(true, 100.0),
            )
            .await;
        }
    }

    let status_code = response.status().as_u16() as i64;
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or_default()
        .to_owned();
    let mut response_headers = BTreeMap::new();
    for (name, value) in response.headers() {
        response_headers.insert(
            name.as_str().to_owned(),
            value.to_str().unwrap_or_default().to_owned(),
        );
    }
    let (mut content_type, mut content_type_encoding) = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(parse_content_type)
        .unwrap_or_default();
    let expected_length = response.content_length().unwrap_or(0);
    {
        let mut current = state.lock().unwrap();
        if current.generation == generation {
            current.response_headers = response_headers.clone();
            current.content_type = content_type.clone();
            current.content_type_encoding = content_type_encoding.clone();
            current.content_length = expected_length;
            if callbacks.is_some() {
                current.ready_state = 3;
            }
        }
    }
    if let Some((executor, object)) = callbacks.as_ref() {
        dispatch_event(
            executor.clone(),
            object.clone(),
            Arc::clone(&state),
            generation,
            HttpEvent::ReadyState(3),
        )
        .await;
    }

    let mut response_data = Vec::new();
    let mut downloaded = 0u64;
    let mut network_error = None;
    if response.status() == reqwest::StatusCode::OK {
        loop {
            let next = tokio::select! {
                _ = cancellation.cancelled() => {
                    finish_request(
                        &state,
                        generation,
                        operation_id,
                        -1,
                        "aborted",
                        None,
                        None,
                        downloaded,
                        callbacks.is_some(),
                    );
                    if let Some((executor, object)) = callbacks {
                        dispatch_event(
                            executor,
                            object,
                            Arc::clone(&state),
                            generation,
                            HttpEvent::ReadyState(4),
                        )
                        .await;
                    }
                    return -1;
                },
                chunk = response.chunk() => chunk,
            };
            match next {
                Ok(Some(chunk)) => {
                    downloaded += chunk.len() as u64;
                    if content_type.eq_ignore_ascii_case("text/html")
                        && content_type_encoding.is_empty()
                    {
                        if let Some(meta_content_type) = html_meta_content_type(&chunk) {
                            let (meta_type, meta_encoding) = parse_content_type(&meta_content_type);
                            if !meta_type.is_empty() {
                                content_type = meta_type;
                                let mut current = state.lock().unwrap();
                                if current.generation == generation {
                                    current.content_type = content_type.clone();
                                    current.content_type_encoding = meta_encoding.clone();
                                }
                                content_type_encoding = meta_encoding;
                            }
                        }
                    }
                    if let Some(writer) = output.as_mut() {
                        if let Err(error) = writer.0.write_all(&chunk) {
                            network_error = Some(error.to_string());
                            break;
                        }
                    } else {
                        response_data.extend_from_slice(&chunk);
                    }
                    if let Some((executor, object)) = callbacks.as_ref() {
                        let percent = if expected_length == 0 {
                            0.0
                        } else {
                            (downloaded as f64 * 100.0 / expected_length as f64).min(100.0)
                        };
                        dispatch_event(
                            executor.clone(),
                            object.clone(),
                            Arc::clone(&state),
                            generation,
                            HttpEvent::Progress(false, percent),
                        )
                        .await;
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    network_error = Some(error.to_string());
                    break;
                }
            }
        }
    }

    if let Some(writer) = output.as_mut() {
        let _ = writer.0.flush();
    }
    let (final_status, final_text) = match network_error {
        Some(message) => (0, message),
        None => (status_code, status_text),
    };
    let stored_data = if output.is_some() {
        None
    } else if response_data.is_empty() {
        None
    } else {
        Some(response_data)
    };
    finish_request(
        &state,
        generation,
        operation_id,
        final_status,
        &final_text,
        stored_data,
        Some((content_type, content_type_encoding)),
        downloaded,
        callbacks.is_some(),
    );
    if let Some((executor, object)) = callbacks {
        dispatch_event(
            executor,
            object,
            Arc::clone(&state),
            generation,
            HttpEvent::ReadyState(4),
        )
        .await;
    }
    final_status
}

fn finish_request(
    state: &Arc<Mutex<RequestState>>,
    generation: u64,
    operation_id: u64,
    status: i64,
    status_text: &str,
    response_data: Option<Vec<u8>>,
    content_type: Option<(String, String)>,
    downloaded: u64,
    asynchronous: bool,
) {
    let mut state = state.lock().unwrap();
    if state.generation != generation || state.operation_id != operation_id {
        return;
    }
    state.status = status;
    state.status_text = status_text.to_owned();
    if let Some(response_data) = response_data {
        state.response_data = Some(response_data);
    }
    if let Some((content_type, encoding)) = content_type {
        state.content_type = content_type;
        state.content_type_encoding = encoding;
    }
    if state.content_length == 0 {
        state.content_length = downloaded;
    }
    state.running = false;
    state.cancellation = None;
    if asynchronous {
        state.ready_state = 4;
    }
}

pub struct HttpRequest {
    state: Arc<Mutex<RequestState>>,
    cert: bool,
    agent_name: String,
}

#[Tjs2Class]
#[tjs(class_name = "HttpRequest")]
impl HttpRequest {
    fn new(
        #[tjs(this)] _this: *mut iTJSDispatch2,
        _window: &mut tTJSVariant,
        cert: Option<i64>,
        agent_name: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(RequestState::default())),
            cert: cert.is_none_or(|value| value != 0),
            agent_name: agent_name.unwrap_or_else(|| "KIRIKIRI".to_owned()),
        })
    }

    #[tjs(case = camel)]
    fn open(
        &self,
        #[tjs(this)] this: *mut iTJSDispatch2,
        method: String,
        url: String,
        user_name: Option<String>,
        password: Option<String>,
    ) -> Result<()> {
        let method =
            reqwest::Method::from_bytes(method.as_bytes()).context("invalid HTTP method")?;
        let mut url = reqwest::Url::parse(&url).context("invalid URL")?;
        if !matches!(url.scheme(), "http" | "https") {
            bail!("unsupported protocol");
        }
        let (auth_user, auth_password) = if !url.username().is_empty() {
            let username = url.username().to_owned();
            let password = url.password().unwrap_or_default().to_owned();
            let _ = url.set_username("");
            let _ = url.set_password(None);
            (Some(username), Some(password))
        } else {
            (user_name, password)
        };
        let client = reqwest::Client::builder()
            .user_agent(&self.agent_name)
            .danger_accept_invalid_certs(!self.cert)
            .build()?;

        let generation = {
            let mut state = self.state.lock().unwrap();
            if let Some(cancellation) = state.cancellation.take() {
                cancellation.cancel();
            }
            state.generation += 1;
            state.running = false;
            state.ready_state = 1;
            state.status = 0;
            state.status_text.clear();
            state.content_type.clear();
            state.content_type_encoding.clear();
            state.content_length = 0;
            state.response_headers.clear();
            state.response_data = None;
            state.pending_open_event = None;
            state.request = Some(RequestSpec {
                client,
                method,
                url,
                headers: Vec::new(),
                encoding: None,
                auth: auth_user.map(|user| (user, auth_password.unwrap_or_default())),
            });
            state.generation
        };
        if !this.is_null() {
            let pending_open_event = schedule_event(
                this,
                Arc::clone(&self.state),
                generation,
                HttpEvent::ReadyState(1),
                true,
            )?;
            self.state.lock().unwrap().pending_open_event = pending_open_event;
        }
        Ok(())
    }

    #[tjs(case = camel)]
    fn set_request_header(&self, name: String, value: String) -> Result<()> {
        let name = HeaderName::from_bytes(name.as_bytes()).context("invalid header name")?;
        let value = HeaderValue::from_str(&value).context("invalid header value")?;
        let mut state = self.state.lock().unwrap();
        if state.running {
            bail!("already running");
        }
        let Some(request) = state.request.as_mut() else {
            return Ok(());
        };
        if name == reqwest::header::CONTENT_TYPE {
            let value = value.to_str().unwrap_or_default();
            let (_, encoding) = parse_content_type(value);
            request.encoding = (!encoding.is_empty()).then_some(encoding);
        }
        request.headers.push((name, value));
        Ok(())
    }

    fn send(
        &self,
        #[tjs(this)] this: *mut iTJSDispatch2,
        data: Option<&mut tTJSVariant>,
        store_storage: Option<String>,
    ) -> Result<()> {
        self.ensure_can_send()?;
        let request_encoding = self
            .state
            .lock()
            .unwrap()
            .request
            .as_ref()
            .ok_or_else(|| anyhow!("not open"))?
            .encoding
            .clone();
        let body = body_from_variant(data, request_encoding.as_deref());
        self.send_body(this, body, store_storage)
    }

    #[tjs(case = camel)]
    fn send_sync(
        &self,
        data: Option<&mut tTJSVariant>,
        store_storage: Option<String>,
    ) -> Result<i64> {
        self.ensure_can_send()?;
        let request_encoding = self
            .state
            .lock()
            .unwrap()
            .request
            .as_ref()
            .ok_or_else(|| anyhow!("not open"))?
            .encoding
            .clone();
        let body = body_from_variant(data, request_encoding.as_deref());
        self.send_body_sync(body, store_storage)
    }

    #[tjs(case = camel)]
    fn send_storage(
        &self,
        #[tjs(this)] this: *mut iTJSDispatch2,
        storage: String,
        store_storage: Option<String>,
    ) -> Result<()> {
        self.ensure_can_send()?;
        let (reader, length) = open_storage_reader(&storage)?;
        self.send_body(this, RequestBody::Storage(reader, length), store_storage)
    }

    #[tjs(case = camel)]
    fn send_storage_sync(&self, storage: String, store_storage: Option<String>) -> Result<i64> {
        self.ensure_can_send()?;
        let (reader, length) = open_storage_reader(&storage)?;
        self.send_body_sync(RequestBody::Storage(reader, length), store_storage)
    }

    #[tjs(skip)]
    fn ensure_can_send(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        if state.running {
            bail!("already running");
        }
        if state.request.is_none() {
            bail!("not open");
        }
        Ok(())
    }

    #[tjs(skip)]
    fn send_body(
        &self,
        this: *mut iTJSDispatch2,
        body: RequestBody,
        store_storage: Option<String>,
    ) -> Result<()> {
        self.ensure_can_send()?;
        let (runtime, executor) = callback_context()?;
        let output = store_storage
            .as_deref()
            .map(open_storage_writer)
            .transpose()?;
        let (request, cancellation, generation, operation_id, output, pending_open_event) =
            begin_request(&self.state, &body, output, true)?;
        let object = HttpRequestTjsObj::new(this);
        let state = Arc::clone(&self.state);
        runtime.spawn(async move {
            run_request(
                state,
                request,
                body,
                output,
                cancellation,
                pending_open_event,
                generation,
                operation_id,
                Some((executor, object)),
            )
            .await;
        });
        Ok(())
    }

    #[tjs(skip)]
    fn send_body_sync(&self, body: RequestBody, store_storage: Option<String>) -> Result<i64> {
        self.ensure_can_send()?;
        let (runtime, _) = callback_context()?;
        let output = store_storage
            .as_deref()
            .map(open_storage_writer)
            .transpose()?;
        let (request, cancellation, generation, operation_id, output, pending_open_event) =
            begin_request(&self.state, &body, output, false)?;
        Ok(runtime.block_on(run_request(
            Arc::clone(&self.state),
            request,
            body,
            output,
            cancellation,
            pending_open_event,
            generation,
            operation_id,
            None,
        )))
    }

    fn abort(&self, #[tjs(this)] this: *mut iTJSDispatch2) {
        let generation = {
            let mut state = self.state.lock().unwrap();
            if !state.running {
                return;
            }
            if let Some(cancellation) = state.cancellation.take() {
                cancellation.cancel();
            }
            state.generation += 1;
            state.running = false;
            state.ready_state = 4;
            state.status = -1;
            state.status_text = "aborted".to_owned();
            state.response_data = None;
            state.generation
        };
        if !this.is_null() {
            let _ = schedule_event(
                this,
                Arc::clone(&self.state),
                generation,
                HttpEvent::ReadyState(4),
                false,
            );
        }
    }

    #[tjs(case = camel)]
    fn get_all_response_headers(&self) -> Result<tTJSVariant> {
        let headers = self.state.lock().unwrap().response_headers.clone();
        Ok(ser::to(&headers)?)
    }

    #[tjs(case = camel)]
    fn get_response_header(&self, name: String) -> tTJSVariant {
        self.state
            .lock()
            .unwrap()
            .response_headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(&name))
            .map(|(_, value)| tTJSVariant::from(value.as_str()))
            .unwrap_or_default()
    }

    #[tjs(case = camel)]
    fn get_response_text(&self, encoding: Option<String>) -> String {
        let state = self.state.lock().unwrap();
        decode_text(
            state.response_data.as_deref().unwrap_or_default(),
            encoding
                .as_deref()
                .or_else(|| Some(state.content_type_encoding.as_str())),
        )
    }

    #[tjs(case = camel)]
    fn get_ready_state(&self) -> i64 {
        self.state.lock().unwrap().ready_state
    }

    fn get_response(&self) -> tTJSVariant {
        let state = self.state.lock().unwrap();
        let Some(data) = state.response_data.as_deref() else {
            return tTJSVariant::new();
        };
        if state.content_type.starts_with("text/") {
            let text = decode_text(data, Some(&state.content_type_encoding));
            tTJSVariant::from(text.as_str())
        } else {
            octet_variant(data)
        }
    }

    #[tjs(case = camel)]
    fn get_response_data(&self) -> tTJSVariant {
        let state = self.state.lock().unwrap();
        octet_variant(state.response_data.as_deref().unwrap_or_default())
    }

    fn get_status(&self) -> i64 {
        self.state.lock().unwrap().status
    }

    #[tjs(case = camel)]
    fn get_status_text(&self) -> String {
        self.state.lock().unwrap().status_text.clone()
    }

    #[tjs(case = camel)]
    fn get_content_type(&self) -> String {
        self.state.lock().unwrap().content_type.clone()
    }

    #[tjs(case = camel)]
    fn get_content_type_encoding(&self) -> String {
        self.state.lock().unwrap().content_type_encoding.clone()
    }

    #[tjs(case = camel)]
    fn get_content_length(&self) -> i64 {
        self.state
            .lock()
            .unwrap()
            .content_length
            .min(i64::MAX as u64) as i64
    }

    #[tjs(static_member, rename = "UNINITIALIZED")]
    fn uninitialized() -> tTJSVariant {
        tTJSVariant::from(0i64)
    }

    #[tjs(static_member, rename = "OPEN")]
    fn open_state() -> tTJSVariant {
        tTJSVariant::from(1i64)
    }

    #[tjs(static_member, rename = "SENT")]
    fn sent_state() -> tTJSVariant {
        tTJSVariant::from(2i64)
    }

    #[tjs(static_member, rename = "RECEIVING")]
    fn receiving_state() -> tTJSVariant {
        tTJSVariant::from(3i64)
    }

    #[tjs(static_member, rename = "LOADED")]
    fn loaded_state() -> tTJSVariant {
        tTJSVariant::from(4i64)
    }
}
