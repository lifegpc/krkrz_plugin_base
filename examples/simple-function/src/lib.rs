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
static mut ORIGIN_SAY_HELLO: *mut iTJSDispatch2 = std::ptr::null_mut();

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let global = unsafe { TVPGetScriptDispatch() };
    let name = ttstr::from("say_hello");
    let n = name.c_str();
    if !global.is_null() {
        let mut val = tTJSVariant::new();
        if TJS_SUCCEEDED(unsafe {
            (*global).prop_get(0, n, std::ptr::null_mut(), &mut val, global)
        }) {
            unsafe {
                ORIGIN_SAY_HELLO = val.as_object();
            }
            val.clear();
        }
        let fns = tTJSDispatch::new(MyFunction {});
        let val = tTJSVariant::from(fns);
        unsafe { (*fns).release() };
        unsafe {
            (*global).prop_set(
                TJS_MEMBERENSURE as u32,
                n,
                std::ptr::null_mut(),
                &val,
                global,
            )
        };
        unsafe { (*global).release() };
    }
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[simple-function]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    let name = ttstr::from("say_hello");
    let n = name.c_str();
    let global = unsafe { TVPGetScriptDispatch() };
    if !global.is_null() {
        unsafe { (*global).delete_member(0, n, std::ptr::null_mut(), global) };
        unsafe {
            if !ORIGIN_SAY_HELLO.is_null() {
                let val: tTJSVariant = tTJSVariant::from(ORIGIN_SAY_HELLO);
                (*ORIGIN_SAY_HELLO).release();
                ORIGIN_SAY_HELLO = std::ptr::null_mut();
                (*global).prop_set(
                    TJS_MEMBERENSURE as u32,
                    n,
                    std::ptr::null_mut(),
                    &val,
                    global,
                );
            }
            (*global).release();
        }
    }
    unsafe { TVPUninitImportStub() };
    0
}
