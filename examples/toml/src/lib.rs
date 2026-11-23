use std::io::{Read, Write};

use krkrz_plugin_base::{tp_stub::*, *};

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
generate_origin_static_block!(toml);

struct TOML;

#[Tjs2Class]
impl TOML {
    #[tjs(static_method)]
    fn parse(json: String) -> Result<tTJSVariant, toml::de::Error> {
        Ok(toml::from_str(&json)?)
    }
    #[tjs(static_method)]
    fn stringify(obj: Option<&mut tTJSVariant>) -> Result<String, toml::ser::Error> {
        Ok(match obj {
            Some(obj) => toml::to_string(obj)?,
            None => toml::to_string(&())?,
        })
    }
    #[tjs(static_method, case = camel)]
    fn eval_storage(file: String) -> anyhow::Result<tTJSVariant> {
        let mut f = match create_istream(&file, TJS_BS_READ) {
            Some(f) => com::IStreamWrapper(f),
            None => {
                anyhow::bail!("Failed to open file: {}", file);
            }
        };
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        Ok(toml::from_str(&s)?)
    }
    #[tjs(static_method)]
    fn save(file: String, obj: Option<&mut tTJSVariant>) -> anyhow::Result<()> {
        let s = Self::stringify(obj)?;
        let mut f = match create_istream(&file, TJS_BS_WRITE) {
            Some(f) => com::IStreamWrapper(f),
            None => {
                anyhow::bail!("Failed to create file: {}", file);
            }
        };
        f.write_all(s.as_bytes())?;
        Ok(())
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system-unwind" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let toml = TOML::create_native_class().1;
    register_var!(case = constant, toml);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system-unwind" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[toml]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(case = constant, toml);
    unsafe { TVPUninitImportStub() };
    0
}
