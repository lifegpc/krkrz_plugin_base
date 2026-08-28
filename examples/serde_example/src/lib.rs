use anyhow::Result;
use krkrz_plugin_base::{de::*, tp_stub::*, *};
use serde::Deserialize;

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
generate_origin_static_block!(a_class, json);

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Version {
    major: i32,
    minor: i32,
    patch: i32,
    full: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
enum Test {
    Pure,
    Simple(u32),
    Tuple(String, u64),
    Struct { test: String },
}

struct AClass {}

#[Tjs2Class]
impl AClass {
    fn new() -> Self {
        Self {}
    }
    fn v1(&self, v: &mut tTJSVariant) -> Result<()> {
        let de = TjsDeserializer::new(v);
        let v = <(String, i64)>::deserialize(de)?;
        log!("{:?}", v);
        Ok(())
    }
    fn v2(&self, v: &mut tTJSVariant) -> Result<()> {
        let de = TjsDeserializer::new(v);
        let v = Version::deserialize(de)?;
        log!("{:?}", v);
        Ok(())
    }
    fn v3(&self, v: &mut tTJSVariant) -> Result<()> {
        let de = TjsDeserializer::new(v);
        let v = Test::deserialize(de)?;
        log!("{:?}", v);
        Ok(())
    }

    #[tjs(serde)]
    fn v4(&self, v: Version, optional: Option<Test>) {
        log!("{:?} {:?}", v, optional);
    }

    fn v5(&self, value: i64, #[tjs(serde)] v: Version) {
        log!("{} {:?}", value, v);
    }
}

struct JSON;

#[Tjs2Class]
impl JSON {
    #[tjs(static_method)]
    fn parse(json: String) -> Result<tTJSVariant> {
        Ok(serde_json::from_str(&json)?)
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let a_class = AClass::create_native_class().1;
    let json = JSON::create_native_class().1;
    register_var!(a_class, case = constant, json);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[serde-example]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(a_class, case = constant, json);
    unsafe { TVPUninitImportStub() };
    0
}
