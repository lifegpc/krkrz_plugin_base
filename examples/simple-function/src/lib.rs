use krkrz_plugin_base::{tp_stub::*, *};

#[tjs2_function]
fn say_hello(name: String) -> String {
    log!("name from func_call: {name}");
    format!("Hello, {name}.")
}

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
generate_origin_static_block!(say_hello);

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let say_hello = create_say_hello();
    register_var!(case = snake, say_hello);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[simple-function]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(case = snake, say_hello);
    unsafe { TVPUninitImportStub() };
    0
}
