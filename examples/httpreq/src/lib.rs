use anyhow::Result;
use krkrz_plugin_base::{tp_stub::*, *};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ptr;
use tokio::runtime::{Builder, Runtime};

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
static mut RUNTIME: Option<Runtime> = None;
static mut EXECUTOR: Option<AsyncExecutor> = None;

generate_origin_static_block!(http_req);

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

struct SendCallbacks {
    callback: tTJSVariantClosure,
    onerror: Option<tTJSVariantClosure>,
}

// SAFETY: `callback` and `onerror` have their refcounts incremented on the
// krkrz window thread before being wrapped in `SendCallbacks`, and are only
// ever accessed, invoked, and released inside `AsyncExecutor::execute` on the
// krkrz window thread.
unsafe impl Send for SendCallbacks {}

impl SendCallbacks {
    /// Must only be called on the krkrz window thread.
    unsafe fn invoke_and_release(mut self, result: Result<OwnedResponse>) {
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

        self.callback.release();
        if let Some(mut onerror) = self.onerror.take() {
            onerror.release();
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case", default)]
struct GetConfig {
    #[serde(alias = "response_type")]
    pub res_type: ResponseType,
    pub no_error_handle: bool,
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

#[Tjs2Class]
impl HttpReq {
    #[tjs(static_method)]
    fn get(
        url: String,
        #[tjs(serde)] config: Option<GetConfig>,
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

        let config = config.unwrap_or_default();
        let res_type = config.res_type;
        let no_error_handle = config.no_error_handle;

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
        };
        unsafe {
            TVPPluginGlobalRefCount += 1;
        }

        runtime.spawn(async move {
            let result: Result<OwnedResponse> = async {
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
            }
            .await;

            executor
                .execute(move || unsafe {
                    callbacks.invoke_and_release(result);
                })
                .await;
        });

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
    register_var!(http_req);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[httpreq]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(http_req);
    unsafe {
        RUNTIME = None;
        EXECUTOR = None;
        TVPUninitImportStub();
    }
    0
}
