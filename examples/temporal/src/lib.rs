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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Temporal(t) => t.fmt(f),
            Self::Type(t) => t.fmt(f),
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
    fn from(info: &'static mut tTJSVariant) -> Result<Self, Error> {
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
                let obj = unsafe { &mut *obj };
                let hr =
                    unsafe { obj.native_instance_support(2, CID_INSTANT, std::ptr::null_mut()) };
                if TJS_FAILED(hr) {
                    return Err(TypeError("Instant").into());
                }
                // #TODO: clone via access epochNanoseconds
                unimplemented!()
            }
            _ => Err(TypeError("string or Instant").into()),
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
