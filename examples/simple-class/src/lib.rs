use krkrz_plugin_base::{tp_stub::*, *};

struct SimpleClass {}

impl SimpleClass {
    fn new() -> Self {
        Self {}
    }

    unsafe extern "C" fn new_ni() -> *mut iTJSNativeInstance {
        tTJSNativeInstance::new(Self::new())
    }
}

impl TJSNativeInstance for SimpleClass {
    fn construct(
        &mut self,
        numparams: tjs_int,
        _param: *mut *mut tTJSVariant,
        _tjs_obj: *mut iTJSDispatch2,
    ) -> tjs_error {
        log!("[simple-class]Construct: {numparams}");
        TJS_S_OK
    }
    fn invalidate(&mut self) {
        log!("[simple-class]Deconstruct");
    }
}

unsafe extern "C" fn ncm_finalize(
    _result: *mut tTJSVariant,
    _numparams: tjs_int,
    _param: *mut *mut tTJSVariant,
    _tjs_obj: *mut iTJSDispatch2,
) -> tjs_error {
    log!("[simple-class]ncm_finalize");
    TJS_S_OK
}

static mut SIMPLE_CLASS_CID: tjs_int32 = -1;
static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
generate_origin_static_block!(simple_class);

unsafe extern "C" fn ncm_construct(
    _result: *mut tTJSVariant,
    numparams: tjs_int,
    param: *mut *mut tTJSVariant,
    tjs_obj: *mut iTJSDispatch2,
) -> tjs_error {
    log!("[simple-class]ncm_construct");
    let mut _this: *mut iTJSNativeInstance = std::ptr::null_mut();
    let hr =
        unsafe { (*tjs_obj).native_instance_support(0x00000002, SIMPLE_CLASS_CID, &mut _this) };
    if TJS_FAILED(hr) {
        return hr;
    }
    if _this.is_null() {
        return TJS_E_NATIVECLASSCRASH;
    }
    unsafe { (*_this).construct(numparams, param, tjs_obj) }
}

fn create_native_class() -> *mut iTJSDispatch2 {
    let classname: tTJSString = "SimpleClass".into();
    let classobj =
        unsafe { TJSCreateNativeClassForPlugin(&classname as *const _, Some(SimpleClass::new_ni)) }
            as *mut tTJSNativeClass;
    let name = classname.c_str();
    unsafe { SIMPLE_CLASS_CID = TJSRegisterNativeClass(name) };
    unsafe {
        TJSNativeClassSetClassID(classobj, SIMPLE_CLASS_CID);
    }
    let fnname: ttstr = "finalize".into();
    let fname = fnname.c_str();
    unsafe {
        TJSNativeClassRegisterNCM(
            classobj,
            fname,
            TJSCreateNativeClassMethod(Some(ncm_finalize)) as *mut _,
            name,
            tTJSNativeInstanceType_nitMethod,
            0,
        );
        TJSNativeClassRegisterNCM(
            classobj,
            name,
            TJSCreateNativeClassConstructor(Some(ncm_construct)) as *mut _,
            name,
            tTJSNativeInstanceType_nitClass,
            0,
        );
    }
    classobj as *mut iTJSDispatch2
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let simple_class = create_native_class();
    register_var!(simple_class);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[simple-class]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(simple_class);
    log!("[simple-class]unlinked plugin");
    unsafe { TVPUninitImportStub() };
    0
}
