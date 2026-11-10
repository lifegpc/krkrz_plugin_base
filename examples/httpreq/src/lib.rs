use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use krkrz_plugin_base::{tp_stub::*, *};
use reqwest::header::{CONNECTION, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION, UPGRADE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ptr;
use std::sync::{Arc, Mutex, MutexGuard, TryLockError, TryLockResult};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_util::sync::CancellationToken;

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
static mut RUNTIME: Option<Runtime> = None;
static mut EXECUTOR: Option<AsyncExecutor> = None;

generate_origin_static_block!(http_req, abort_signal, web_socket);

enum ResponseBody {
    Bytes(Vec<u8>),
    Text(String),
    Json(serde_json::Value),
}

struct OwnedResponse {
    status: reqwest::StatusCode,
    version: &'static str,
    headers: reqwest::header::HeaderMap,
    url: String,
    remote_addr: Option<String>,
    body: ResponseBody,
}

impl OwnedResponse {
    fn as_simple_response(&self) -> SimpleResponse<'_> {
        let mut headers = HashMap::with_capacity(self.headers.len());
        for (name, value) in &self.headers {
            let val = match value.to_str() {
                Ok(s) => HeaderValue::Str(s),
                Err(_) => HeaderValue::Bytes(value.as_bytes()),
            };
            headers.insert(name.as_str(), val);
        }

        let (text, bytes, json) = match &self.body {
            ResponseBody::Bytes(b) => (None, Some(b.as_slice()), None),
            ResponseBody::Text(t) => (Some(t.as_str()), None, None),
            ResponseBody::Json(j) => (None, None, Some(j)),
        };

        SimpleResponse {
            ok: self.status.is_success(),
            code: self.status.as_u16(),
            status: self.status.canonical_reason().unwrap_or(""),
            version: self.version,
            headers,
            url: self.url.as_str(),
            remote_addr: self.remote_addr.as_deref(),
            text,
            bytes,
            json,
        }
    }
}

enum RequestOutcome {
    Completed(Result<OwnedResponse>),
    Aborted,
}

struct SendCallbacks {
    callback: tTJSVariantClosure,
    onerror: Option<tTJSVariantClosure>,
    signal_obj: Option<*mut iTJSDispatch2>,
}

// SAFETY: `callback`, `onerror`, and `signal_obj` have their refcounts incremented
// on the krkrz window thread before being wrapped in `SendCallbacks`, and are only
// ever accessed, invoked, and released inside `AsyncExecutor::execute` on the
// krkrz window thread.
unsafe impl Send for SendCallbacks {}

impl SendCallbacks {
    /// Must only be called on the krkrz window thread.
    unsafe fn invoke_and_release(mut self, outcome: RequestOutcome) {
        let is_aborted = matches!(outcome, RequestOutcome::Aborted)
            || self
                .signal_obj
                .is_some_and(|sig_obj| AbortSignal::get_ref(sig_obj).get_aborted());

        if is_aborted {
            if let Some(onerror) = self.onerror.as_mut() {
                let reason_var = self
                    .signal_obj
                    .and_then(|sig_obj| AbortSignal::get_ref(sig_obj).reason.clone())
                    .unwrap_or_else(|| tTJSVariant::from("Aborted"));
                let mut args = Tjs2ArgsArray::new([reason_var]);
                let _ = unsafe {
                    onerror.try_func_call(
                        0,
                        ptr::null(),
                        ptr::null_mut(),
                        ptr::null_mut(),
                        args.len() as tjs_int,
                        args.as_mut_ptr(),
                        ptr::null_mut(),
                    )
                };
            }
        } else if let RequestOutcome::Completed(result) = outcome {
            let variant_res = result.and_then(|resp| {
                let simple = resp.as_simple_response();
                ser::to(&simple).map_err(|e| anyhow::anyhow!("{e}"))
            });

            match variant_res {
                Ok(variant) => {
                    let mut args = Tjs2ArgsArray::new([variant]);
                    let _ = unsafe {
                        self.callback.try_func_call(
                            0,
                            ptr::null(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            args.len() as tjs_int,
                            args.as_mut_ptr(),
                            ptr::null_mut(),
                        )
                    };
                }
                Err(err) => {
                    if let Some(onerror) = self.onerror.as_mut() {
                        let errmsg = err.to_string();
                        let mut args = tjs2_get_args!(errmsg.as_str());
                        let _ = unsafe {
                            onerror.try_func_call(
                                0,
                                ptr::null(),
                                ptr::null_mut(),
                                ptr::null_mut(),
                                args.len() as tjs_int,
                                args.as_mut_ptr(),
                                ptr::null_mut(),
                            )
                        };
                    }
                }
            }
        }

        self.callback.release();
        if let Some(mut onerror) = self.onerror.take() {
            onerror.release();
        }
        if let Some(sig_obj) = self.signal_obj.take() {
            unsafe {
                (*sig_obj).release();
            }
        }
        unsafe {
            TVPPluginGlobalRefCount -= 1;
        }
    }
}

struct HttpReq;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ResponseType {
    Bytes,
    #[default]
    Text,
    Json,
}

fn serialize_bytes_slice<S: serde::Serializer>(
    v: &[u8],
    s: S,
) -> std::result::Result<S::Ok, S::Error> {
    s.serialize_bytes(v)
}

fn serialize_opt_bytes_slice<S: serde::Serializer>(
    v: &Option<&[u8]>,
    s: S,
) -> std::result::Result<S::Ok, S::Error> {
    match v {
        Some(b) => s.serialize_bytes(b),
        None => s.serialize_none(),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
enum HeaderValue<'a> {
    #[serde(serialize_with = "serialize_bytes_slice")]
    Bytes(&'a [u8]),
    Str(&'a str),
}

#[derive(Debug, Clone, Serialize)]
struct SimpleResponse<'a> {
    ok: bool,
    /// Status code
    code: u16,
    /// Status code string
    status: &'a str,
    /// HTTP version string. eg. HTTP/1.1
    version: &'a str,
    headers: HashMap<&'a str, HeaderValue<'a>>,
    url: &'a str,
    remote_addr: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_opt_bytes_slice"
    )]
    bytes: Option<&'a [u8]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    json: Option<&'a serde_json::Value>,
}

struct AbortSignal {
    pub token: CancellationToken,
    pub reason: Option<tTJSVariant>,
}

#[Tjs2Class]
impl AbortSignal {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            reason: None,
        }
    }

    pub fn get_aborted(&self) -> bool {
        self.token.is_cancelled()
    }

    pub fn get_reason(&self) -> tTJSVariant {
        self.reason.clone().unwrap_or_default()
    }

    pub fn abort(&mut self, reason: Option<&mut tTJSVariant>) {
        self.reason = reason.map(|s| s.clone());
        self.token.cancel();
    }
}

#[Tjs2Class]
impl HttpReq {
    #[tjs(static_method)]
    fn get(
        url: String,
        config: Option<&mut tTJSVariant>,
        callback: &mut tTJSVariant,
        onerror: Option<&mut tTJSVariant>,
    ) -> Result<()> {
        if !callback.is_object() {
            anyhow::bail!("callback must be a callable object");
        }
        let onerror_closure = match onerror {
            None => None,
            Some(onerror) if onerror.is_void() => None,
            Some(onerror) if onerror.is_object() => {
                let clo = onerror.as_object_closure();
                Some(tTJSVariantClosure::new(clo.Object, clo.ObjThis))
            }
            Some(_) => anyhow::bail!("onerror must be a callable object or void"),
        };
        let callback_closure = {
            let clo = callback.as_object_closure();
            tTJSVariantClosure::new(clo.Object, clo.ObjThis)
        };
        let config_obj = match config.as_ref() {
            Some(c) => {
                let a = c.as_object_no_add_ref();
                if a.is_null() {
                    anyhow::bail!("config is not a object.");
                }
                Some(unsafe { &mut (*a) })
            }
            None => None,
        };
        let mut res_type = ResponseType::default();
        let mut no_error_handle = false;
        let mut signal = None;
        if let Some(c) = config_obj {
            c.with_get_serde("response_type", |s| {
                res_type = s;
            })?;
            c.with_get_serde("no_error_handle", |s| {
                no_error_handle = s;
            })?;
            signal = c.get("signal")?;
        }
        let (cancel_token, signal_obj) = match signal.as_mut() {
            Some(signal) => {
                if !AbortSignal::is_instance_of2(signal) {
                    anyhow::bail!("signal is not an AbortSignal object.");
                }
                let token = AbortSignal::get_ref2(signal).token.clone();
                (Some(token), Some(signal.as_object()))
            }
            None => (None, None),
        };

        let (runtime, executor) = unsafe {
            let rt = (&raw const RUNTIME)
                .as_ref()
                .and_then(|opt| opt.as_ref())
                .ok_or_else(|| anyhow::anyhow!("tokio runtime is not initialized"))?;
            let exec = (&raw const EXECUTOR)
                .as_ref()
                .and_then(|opt| opt.as_ref())
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("AsyncExecutor is not initialized"))?;
            (rt, exec)
        };

        let callbacks = SendCallbacks {
            callback: callback_closure,
            onerror: onerror_closure,
            signal_obj,
        };
        unsafe {
            TVPPluginGlobalRefCount += 1;
        }

        runtime.spawn(async move {
            let req_fut = async {
                let response = reqwest::get(&url).await?;
                let response = if no_error_handle {
                    response
                } else {
                    response.error_for_status()?
                };
                let status = response.status();
                let version = match response.version() {
                    reqwest::Version::HTTP_09 => "HTTP/0.9",
                    reqwest::Version::HTTP_10 => "HTTP/1.0",
                    reqwest::Version::HTTP_11 => "HTTP/1.1",
                    reqwest::Version::HTTP_2 => "HTTP/2.0",
                    reqwest::Version::HTTP_3 => "HTTP/3.0",
                    _ => "HTTP/1.1",
                };
                let headers = response.headers().clone();
                let resp_url = response.url().to_string();
                let remote_addr = response.remote_addr().map(|addr| addr.to_string());

                let body = match res_type {
                    ResponseType::Bytes => {
                        let bytes = response.bytes().await?;
                        ResponseBody::Bytes(bytes.to_vec())
                    }
                    ResponseType::Text => {
                        let text = response.text().await?;
                        ResponseBody::Text(text)
                    }
                    ResponseType::Json => {
                        let bytes = response.bytes().await?;
                        let json = serde_json::from_slice::<serde_json::Value>(&bytes)?;
                        ResponseBody::Json(json)
                    }
                };

                Ok(OwnedResponse {
                    status,
                    version,
                    headers,
                    url: resp_url,
                    remote_addr,
                    body,
                })
            };

            let outcome = match cancel_token {
                Some(token) => match token.run_until_cancelled(req_fut).await {
                    Some(res) => RequestOutcome::Completed(res),
                    None => RequestOutcome::Aborted,
                },
                None => RequestOutcome::Completed(req_fut.await),
            };

            executor
                .execute(move || unsafe {
                    callbacks.invoke_and_release(outcome);
                })
                .await;
        });

        Ok(())
    }
}

struct AsyncMutexGuard<T, F> {
    inner: Arc<Mutex<T>>,
    fut: Option<F>,
    sleep: std::pin::Pin<Box<tokio::time::Sleep>>,
    interval: std::time::Duration,
}

impl<T, F, R> Future for AsyncMutexGuard<T, F>
where
    F: for<'a> FnOnce(TryLockResult<MutexGuard<'a, T>>) -> R + std::marker::Unpin,
{
    type Output = R;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();

        match this.inner.try_lock() {
            Err(TryLockError::WouldBlock) => {
                if this.sleep.as_mut().poll(cx).is_ready() {
                    let next_deadline = tokio::time::Instant::now() + this.interval;
                    this.sleep.as_mut().reset(next_deadline);
                    let _ = this.sleep.as_mut().poll(cx);
                }
                std::task::Poll::Pending
            }
            other => {
                let fut = this
                    .fut
                    .take()
                    .expect("AsyncMutexGuard polled after completion");
                std::task::Poll::Ready(fut(other))
            }
        }
    }
}
trait MutexExt<T> {
    fn lock_async<F, R>(&self, f: F) -> AsyncMutexGuard<T, F>
    where
        F: for<'a> FnOnce(TryLockResult<MutexGuard<'a, T>>) -> R + std::marker::Unpin;

    fn lock_async_delayed<F, R>(
        &self,
        interval: std::time::Duration,
        f: F,
    ) -> AsyncMutexGuard<T, F>
    where
        F: for<'a> FnOnce(TryLockResult<MutexGuard<'a, T>>) -> R + std::marker::Unpin;
}

impl<T> MutexExt<T> for Arc<Mutex<T>> {
    fn lock_async<F, R>(&self, f: F) -> AsyncMutexGuard<T, F>
    where
        F: for<'a> FnOnce(TryLockResult<MutexGuard<'a, T>>) -> R + std::marker::Unpin,
    {
        self.lock_async_delayed(std::time::Duration::from_millis(5), f)
    }

    fn lock_async_delayed<F, R>(&self, interval: std::time::Duration, f: F) -> AsyncMutexGuard<T, F>
    where
        F: for<'a> FnOnce(TryLockResult<MutexGuard<'a, T>>) -> R + std::marker::Unpin,
    {
        AsyncMutexGuard {
            inner: self.clone(),
            fut: Some(f),
            sleep: Box::pin(tokio::time::sleep(interval)),
            interval,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "snake_case")]
enum WebSocketReadyState {
    #[default]
    Connecting,
    Open,
    Closing,
    Closed,
}

enum WsCommand {
    Send(Message),
    Flush,
    Close(Option<CloseFrame>),
}

struct WebSocketState {
    sender: Option<UnboundedSender<WsCommand>>,
    ready_state: WebSocketReadyState,
}

struct WebSocketTjsObj(*mut iTJSDispatch2);

impl WebSocketTjsObj {
    pub fn new(obj: *mut iTJSDispatch2) -> Self {
        if !obj.is_null() {
            unsafe { (*obj).add_ref() };
        }
        Self(obj)
    }

    pub fn on_open(&self) {
        if self.0.is_null() {
            return;
        }
        let t = unsafe { &mut (*self.0) };
        let mut val = tTJSVariant::new();
        let re = unsafe { t.prop_get(0, tjs_w!("onopen"), ptr::null_mut(), &mut val, self.0) };
        if TJS_FAILED(re) || !val.is_object() {
            return;
        }
        let v = val.as_object_closure();
        let mut v = tTJSVariantClosure::new(v.Object, v.ObjThis);
        let _ = unsafe {
            v.try_func_call(
                0,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        v.release();
    }

    pub fn on_message(&self, msg: tTJSVariant) {
        if self.0.is_null() {
            return;
        }
        let t = unsafe { &mut (*self.0) };
        let mut val = tTJSVariant::new();
        let re = unsafe { t.prop_get(0, tjs_w!("onmessage"), ptr::null_mut(), &mut val, self.0) };
        if TJS_FAILED(re) || !val.is_object() {
            return;
        }
        let v = val.as_object_closure();
        let mut v = tTJSVariantClosure::new(v.Object, v.ObjThis);
        let mut args = Tjs2ArgsArray::new([msg]);
        let _ = unsafe {
            v.try_func_call(
                0,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                args.len() as i32,
                args.as_mut_ptr(),
                ptr::null_mut(),
            )
        };
        v.release();
    }

    pub fn on_close(&self, code: u16, reason: String) {
        if self.0.is_null() {
            return;
        }
        let t = unsafe { &mut (*self.0) };
        let mut val = tTJSVariant::new();
        let re = unsafe { t.prop_get(0, tjs_w!("onclose"), ptr::null_mut(), &mut val, self.0) };
        if TJS_FAILED(re) || !val.is_object() {
            return;
        }
        let v = val.as_object_closure();
        let mut v = tTJSVariantClosure::new(v.Object, v.ObjThis);
        let mut args = tjs2_get_args!(code as i64, reason);
        let _ = unsafe {
            v.try_func_call(
                0,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                args.len() as i32,
                args.as_mut_ptr(),
                ptr::null_mut(),
            )
        };
        v.release();
    }

    pub fn on_error(&self, e: String) {
        if self.0.is_null() {
            return;
        }
        let t = unsafe { &mut (*self.0) };
        let mut val = tTJSVariant::new();
        let re = unsafe { t.prop_get(0, tjs_w!("onerror"), ptr::null_mut(), &mut val, self.0) };
        if TJS_FAILED(re) || !val.is_object() {
            return;
        }
        let v = val.as_object_closure();
        let mut v = tTJSVariantClosure::new(v.Object, v.ObjThis);
        let mut args = tjs2_get_args!(e);
        let _ = unsafe {
            v.try_func_call(
                0,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                args.len() as i32,
                args.as_mut_ptr(),
                ptr::null_mut(),
            )
        };
        v.release();
    }
}

impl Drop for WebSocketTjsObj {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { (*self.0).release() };
        }
    }
}

impl Clone for WebSocketTjsObj {
    fn clone(&self) -> Self {
        if !self.0.is_null() {
            unsafe { (*self.0).add_ref() };
        }
        Self(self.0)
    }
}

unsafe impl Send for WebSocketTjsObj {}

struct WebSocket {
    state: Arc<Mutex<WebSocketState>>,
    url: String,
}

impl Drop for WebSocket {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.state.lock() {
            guard.sender = None;
        }
    }
}

fn convert_ws_to_http_url(raw_url: &str) -> Result<reqwest::Url, anyhow::Error> {
    let mut url = reqwest::Url::parse(raw_url)?;
    match url.scheme() {
        "ws" => {
            let _ = url.set_scheme("http");
        }
        "wss" => {
            let _ = url.set_scheme("https");
        }
        _ => {}
    }
    Ok(url)
}

#[Tjs2Class]
impl WebSocket {
    fn new(#[tjs(this)] tjs_this: *mut iTJSDispatch2, url: String) -> Result<Self> {
        let state = Arc::new(Mutex::new(WebSocketState {
            sender: None,
            ready_state: WebSocketReadyState::Connecting,
        }));
        let this = Self {
            state: state.clone(),
            url: url.clone(),
        };
        let rt = unsafe {
            (&raw const RUNTIME)
                .as_ref()
                .and_then(|opt| opt.as_ref())
                .ok_or_else(|| anyhow::anyhow!("tokio runtime is not initialized"))?
        };
        let rt_handle = rt.handle().clone();
        let exec = unsafe {
            (&raw const EXECUTOR)
                .as_ref()
                .and_then(|opt| opt.as_ref())
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("AsyncExecutor is not initialized"))?
        };
        let tjs_this = WebSocketTjsObj::new(tjs_this);
        rt.spawn(async move {
            let fut = async {
                let key = tokio_tungstenite::tungstenite::handshake::client::generate_key();
                let client = reqwest::Client::builder().http1_only().build()?;
                let response = client
                    .get(convert_ws_to_http_url(&url)?)
                    .header(CONNECTION, "Upgrade")
                    .header(UPGRADE, "websocket")
                    .header(SEC_WEBSOCKET_VERSION, "13")
                    .header(SEC_WEBSOCKET_KEY, key)
                    .send()
                    .await?;
                if response.status() != reqwest::StatusCode::SWITCHING_PROTOCOLS {
                    anyhow::bail!(
                        "Failed to upgrade protocols to websocket: {}",
                        response.status()
                    );
                }
                let upgraded_stream = response.upgrade().await?;
                Ok(WebSocketStream::from_raw_socket(
                    upgraded_stream,
                    tokio_tungstenite::tungstenite::protocol::Role::Client,
                    None,
                )
                .await)
            };
            let re = fut.await;
            match re {
                Err(e) => {
                    state
                        .lock_async(|f| {
                            f.unwrap().ready_state = WebSocketReadyState::Closed;
                        })
                        .await;
                    exec.execute(move || {
                        tjs_this.on_error(e.to_string());
                    })
                    .await;
                }
                Ok(s) => {
                    let (mut sink, mut stream) = s.split();
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WsCommand>();
                    let tx_for_stream = tx.clone();

                    let should_open = state
                        .lock_async(|f| {
                            let mut f = f.unwrap();
                            if matches!(
                                f.ready_state,
                                WebSocketReadyState::Closing | WebSocketReadyState::Closed
                            ) {
                                f.ready_state = WebSocketReadyState::Closed;
                                false
                            } else {
                                f.sender = Some(tx);
                                f.ready_state = WebSocketReadyState::Open;
                                true
                            }
                        })
                        .await;

                    if !should_open {
                        let _ = sink.close().await;
                        exec.execute(move || {
                            tjs_this.on_close(1005, String::new());
                        })
                        .await;
                        return;
                    }

                    let tjs_open = tjs_this.clone();
                    exec.execute(move || {
                        tjs_open.on_open();
                    })
                    .await;

                    let tjs_write_err = tjs_this.clone();
                    let exec_write = exec.clone();
                    rt_handle.spawn(async move {
                        while let Some(cmd) = rx.recv().await {
                            match cmd {
                                WsCommand::Send(msg) => {
                                    if let Err(e) = sink.send(msg).await {
                                        let err_str = e.to_string();
                                        let tjs_e = tjs_write_err.clone();
                                        exec_write
                                            .execute(move || {
                                                tjs_e.on_error(err_str);
                                            })
                                            .await;
                                        break;
                                    }
                                }
                                WsCommand::Flush => {
                                    if let Err(e) = sink.flush().await {
                                        let err_str = e.to_string();
                                        let tjs_e = tjs_write_err.clone();
                                        exec_write
                                            .execute(move || {
                                                tjs_e.on_error(err_str);
                                            })
                                            .await;
                                        break;
                                    }
                                }
                                WsCommand::Close(frame) => {
                                    let _ = sink.send(Message::Close(frame)).await;
                                    let _ = sink.close().await;
                                    break;
                                }
                            }
                        }
                    });

                    let mut close_info: Option<(u16, String)> = None;
                    while let Some(item) = stream.next().await {
                        match item {
                            Ok(Message::Text(text)) => {
                                let msg_str = text.to_string();
                                let tjs_msg = tjs_this.clone();
                                exec.execute(move || {
                                    tjs_msg.on_message(tTJSVariant::from(msg_str.as_str()));
                                })
                                .await;
                            }
                            Ok(Message::Binary(bytes)) => {
                                let vec = bytes.to_vec();
                                let tjs_msg = tjs_this.clone();
                                exec.execute(move || {
                                    let var =
                                        tTJSVariant::from((vec.as_ptr(), vec.len() as tjs_uint));
                                    tjs_msg.on_message(var);
                                })
                                .await;
                            }
                            Ok(Message::Close(frame)) => {
                                close_info = Some(match frame {
                                    Some(f) => (u16::from(f.code), f.reason.to_string()),
                                    None => (1005, String::new()),
                                });
                                let _ = tx_for_stream.send(WsCommand::Close(None));
                                break;
                            }
                            Ok(Message::Ping(_)) => {
                                let _ = tx_for_stream.send(WsCommand::Flush);
                            }
                            Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {}
                            Err(e) => {
                                let err_str = e.to_string();
                                let tjs_err = tjs_this.clone();
                                exec.execute(move || {
                                    tjs_err.on_error(err_str);
                                })
                                .await;
                                break;
                            }
                        }
                    }

                    drop(tx_for_stream);
                    state
                        .lock_async(|f| {
                            let mut f = f.unwrap();
                            f.sender = None;
                            f.ready_state = WebSocketReadyState::Closed;
                        })
                        .await;

                    let (code, reason) = close_info.unwrap_or((1005, String::new()));
                    exec.execute(move || {
                        tjs_this.on_close(code, reason);
                    })
                    .await;
                }
            }
        });
        Ok(this)
    }

    fn get_url(&self) -> &str {
        &self.url
    }

    #[tjs(case = camel)]
    fn get_ready_state(&self) -> Result<tTJSVariant> {
        Ok(ser::to(&self.state.lock().unwrap().ready_state)?)
    }

    fn send(&self, data: &mut tTJSVariant) -> Result<()> {
        let msg = if data.is_string() {
            let s = String::to_param(data).map_err(|e| anyhow::anyhow!("{e}"))?;
            Message::Text(s.into())
        } else if data.is_octet() {
            let oct = types::Octet::to_param(data).map_err(|e| anyhow::anyhow!("{e}"))?;
            Message::Binary(oct.to_vec().into())
        } else {
            anyhow::bail!("WebSocket.send only supports String or Octet");
        };
        let guard = self.state.lock().unwrap();
        if !matches!(guard.ready_state, WebSocketReadyState::Open) {
            anyhow::bail!("WebSocket is not open");
        }
        if let Some(sender) = &guard.sender {
            sender
                .send(WsCommand::Send(msg))
                .map_err(|_| anyhow::anyhow!("WebSocket stream is closed"))?;
        } else {
            anyhow::bail!("WebSocket stream is not available");
        }
        Ok(())
    }

    fn close(&self, code: Option<i64>, reason: Option<String>) -> Result<()> {
        let frame = match (code, reason) {
            (None, None) => None,
            (code_opt, reason_opt) => {
                let code_u16 = match code_opt {
                    Some(c) => u16::try_from(c)
                        .map_err(|_| anyhow::anyhow!("Invalid WebSocket close code: {c}"))?,
                    None => 1000,
                };
                Some(CloseFrame {
                    code: CloseCode::from(code_u16),
                    reason: reason_opt.unwrap_or_default().into(),
                })
            }
        };
        let mut guard = self.state.lock().unwrap();
        match guard.ready_state {
            WebSocketReadyState::Connecting => {
                guard.ready_state = WebSocketReadyState::Closing;
            }
            WebSocketReadyState::Open => {
                guard.ready_state = WebSocketReadyState::Closing;
                if let Some(sender) = guard.sender.take() {
                    let _ = sender.send(WsCommand::Close(frame));
                }
            }
            WebSocketReadyState::Closing | WebSocketReadyState::Closed => {}
        }
        Ok(())
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };

    let runtime = match Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(err) => {
            log!("[httpreq] Failed to create tokio runtime: {err}");
            unsafe { TVPUninitImportStub() };
            return TJS_E_FAIL;
        }
    };
    let executor = AsyncExecutor::new();
    unsafe {
        RUNTIME = Some(runtime);
        EXECUTOR = Some(executor);
    }

    let http_req = HttpReq::create_native_class().1;
    let abort_signal = AbortSignal::create_native_class().1;
    let web_socket = WebSocket::create_native_class().1;
    register_var!(http_req, abort_signal, web_socket);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[httpreq]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(http_req, abort_signal, web_socket);
    unsafe {
        RUNTIME = None;
        EXECUTOR = None;
        TVPUninitImportStub();
    }
    0
}
