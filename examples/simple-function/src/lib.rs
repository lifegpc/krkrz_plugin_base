use krkrz_plugin_base::{tp_stub::*, *};

struct MyFunction {}

impl TJSDispatch for MyFunction {
    fn func_call(
        &mut self,
        _flag: tjs_uint32,
        membername: *const tjs_char,
        _hint: *mut tjs_uint32,
        result: *mut tTJSVariant,
        numparams: tjs_int,
        param: *mut *mut tTJSVariant,
        _objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        if !membername.is_null() {
            return TJS_E_MEMBERNOTFOUND;
        }
        if numparams < 1 {
            return TJS_E_BADPARAMCOUNT;
        }
        if param.is_null() {
            throw_null_access();
        }
        let param0 = unsafe { *param };
        if param0.is_null() {
            throw_null_access();
        }
        let p0 = unsafe { &mut *param0 };
        let s = p0.as_string();
        if s.is_null() {
            return TJS_E_INVALIDPARAM;
        }
        let s = ttstr::from(s);
        let name = s.to_string();
        log!("name from func_call: {name}");
        if !result.is_null() {
            let re = unsafe { &mut *result };
            re.assign(&ttstr::from(&format!("Hello, {name}.")));
        }
        TJS_S_OK
    }
}

impl Drop for MyFunction {
    fn drop(&mut self) {
        log!("Dropped: MyFunction");
    }
}

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
generate_origin_static_block!(say_hello);

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let say_hello = tTJSDispatch::new(MyFunction {});
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
