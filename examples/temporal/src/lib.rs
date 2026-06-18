use std::str::FromStr;

use krkrz_plugin_base::param::TypeError;
use krkrz_plugin_base::{tp_stub::*, *};
use temporal_rs::TemporalError;

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
generate_origin_static_block!(temporal);

struct Instant(temporal_rs::Instant);

enum Error {
    Temporal(TemporalError),
    Type(TypeError),
    Error(String),
}

impl From<TemporalError> for Error {
    fn from(value: TemporalError) -> Self {
        Self::Temporal(value)
    }
}

impl From<TypeError> for Error {
    fn from(value: TypeError) -> Self {
        Self::Type(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Error(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Error(value.to_owned())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Temporal(t) => t.fmt(f),
            Self::Type(t) => t.fmt(f),
            Self::Error(t) => t.fmt(f),
        }
    }
}

#[Tjs2Class]
#[tjs(new = "Temporal.Instant")]
impl Instant {
    fn new(nanoseconds: i128) -> Result<Self, TemporalError> {
        Ok(Self(temporal_rs::Instant::try_new(nanoseconds)?))
    }

    #[allow(non_upper_case_globals)]
    fn from(info: &mut tTJSVariant) -> Result<Self, Error> {
        match info.typ() {
            tTJSVariantType_tvtString => {
                let s = String::to_param(info)?;
                Ok(Self(temporal_rs::Instant::from_str(&s)?))
            }
            tTJSVariantType_tvtObject => {
                let obj = info.as_object_no_add_ref();
                if obj.is_null() {
                    throw_null_access();
                }
                if unsafe {
                    (*obj).is_instance_of(
                        0,
                        std::ptr::null(),
                        std::ptr::null_mut(),
                        tjs_w!("Date"),
                        obj,
                    )
                } == TJS_S_TRUE
                {
                    // is Date
                    let mut var = tTJSVariant::new();
                    let hr = unsafe {
                        (*obj).func_call(
                            0,
                            tjs_w!("getTime"),
                            std::ptr::null_mut(),
                            &mut var,
                            0,
                            std::ptr::null_mut(),
                            obj,
                        )
                    };
                    if TJS_FAILED(hr) {
                        return Err("failed to get time from Date.".into());
                    }
                    let time = i64::to_param(&mut var)?;
                    return Ok(Self(temporal_rs::Instant::from_epoch_milliseconds(time)?));
                }
                let hr = unsafe {
                    (*obj).is_instance_of(
                        0,
                        std::ptr::null(),
                        std::ptr::null_mut(),
                        tjs_w!("Instant"),
                        obj,
                    )
                };
                if hr != TJS_S_TRUE {
                    return Err(TypeError("Instant or Date").into());
                }
                let mut nano = tTJSVariant::new();
                let hr = unsafe {
                    (*obj).prop_get(
                        0,
                        tjs_w!("epochNanoseconds"),
                        std::ptr::null_mut(),
                        &mut nano,
                        obj,
                    )
                };
                if TJS_FAILED(hr) {
                    return Err("failed to get nanoseconds from Instant.".into());
                }
                let nanoseconds = i128::to_param(&mut nano)?;
                Ok(Self(temporal_rs::Instant::try_new(nanoseconds)?))
            }
            _ => Err(TypeError("string or Instant or Date").into()),
        }
    }

    #[tjs(case = camel)]
    fn from_epoch_milliseconds(epoch_milliseconds: i64) -> Result<Self, TemporalError> {
        Ok(Self(temporal_rs::Instant::from_epoch_milliseconds(
            epoch_milliseconds,
        )?))
    }

    #[tjs(case = camel)]
    fn from_epoch_nanoseconds(epoch_nanoseconds: i128) -> Result<Self, TemporalError> {
        Ok(Self(temporal_rs::Instant::try_new(epoch_nanoseconds)?))
    }

    #[tjs(case = camel)]
    fn get_epoch_milliseconds(&self) -> i64 {
        self.0.epoch_milliseconds()
    }

    #[tjs(case = camel)]
    fn get_epoch_nanoseconds(&self) -> i128 {
        self.0.epoch_nanoseconds().0
    }
}

impl Drop for Instant {
    fn drop(&mut self) {
        log!("Instant: dropped {:?}", self.0)
    }
}

struct Temporal {}

#[Tjs2Class]
impl Temporal {
    #[tjs(static_member, case = pascal)]
    fn instant() -> tTJSVariant {
        let class = Instant::create_native_class().1;
        let re = tTJSVariant::from(class);
        unsafe { (*class).release() };
        re
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let temporal = Temporal::create_native_class().1;
    register_var!(temporal);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[temporal]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(temporal);
    log!("[temporal]unlinked plugin");
    unsafe { TVPUninitImportStub() };
    0
}
