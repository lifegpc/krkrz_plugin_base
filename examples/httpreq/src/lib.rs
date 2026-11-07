use anyhow::Result;
use krkrz_plugin_base::{tp_stub::*, *};
use serde::Deserialize;
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
    unsafe fn invoke_and_release(mut self, result: Result<ResponseBody>) {
        let variant_res = result.and_then(|body| match body {
            ResponseBody::Bytes(bytes) => {
                Ok(tTJSVariant::from((bytes.as_ptr(), bytes.len() as tjs_uint)))
            }
            ResponseBody::Text(text) => Ok(tTJSVariant::from(text.as_str())),
            ResponseBody::Json(json) => ser::to(&json).map_err(|e| anyhow::anyhow!("{e}")),
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
pub enum ResponseType {
    Bytes,
    #[default]
    Text,
    Json,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case", default)]
pub struct GetConfig {
    #[serde(alias = "response_type")]
    pub res_type: ResponseType,
}

#[Tjs2Class]
impl HttpReq {
    #[tjs(static_method)]
    pub fn get(
        url: String,
        #[tjs(serde)] config: Option<GetConfig>,
        callback: &mut tTJSVariant,
        onerror: &mut tTJSVariant,
    ) -> Result<()> {
        if !callback.is_object() {
            anyhow::bail!("callback must be a callable object");
        }
        let onerror_closure = if onerror.is_void() {
            None
        } else if onerror.is_object() {
            let clo = onerror.as_object_closure();
            Some(tTJSVariantClosure::new(clo.Object, clo.ObjThis))
        } else {
            anyhow::bail!("onerror must be a callable object or void");
        };
        let callback_closure = {
            let clo = callback.as_object_closure();
            tTJSVariantClosure::new(clo.Object, clo.ObjThis)
        };

        let res_type = config.unwrap_or_default().res_type;

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
            let result: Result<ResponseBody> = async {
                let response = reqwest::get(&url).await?.error_for_status()?;
                match res_type {
                    ResponseType::Bytes => {
                        let bytes = response.bytes().await?;
                        Ok(ResponseBody::Bytes(bytes.to_vec()))
                    }
                    ResponseType::Text => {
                        let text = response.text().await?;
                        Ok(ResponseBody::Text(text))
                    }
                    ResponseType::Json => {
                        let bytes = response.bytes().await?;
                        let json = serde_json::from_slice::<serde_json::Value>(&bytes)?;
                        Ok(ResponseBody::Json(json))
                    }
                }
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
