use anyhow::anyhow;
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

impl Drop for SimpleClass {
    fn drop(&mut self) {
        log!("[simple-class]Dropped")
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
        if numparams == 1 { TJS_E_FAIL } else { TJS_S_OK }
    }
    fn invalidate(&mut self) {
        log!("[simple-class]invalidate");
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
generate_origin_static_block!(simple_class, n_class, my_point);

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
        return TJS_E_NATIVECLASSCRASH;
    }
    if _this.is_null() {
        return TJS_E_NATIVECLASSCRASH;
    }
    unsafe { (*_this).construct(numparams, param, tjs_obj) }
}

unsafe extern "C" fn ncm_from(
    result: *mut tTJSVariant,
    _numparams: tjs_int,
    _param: *mut *mut tTJSVariant,
    _tjs_obj: *mut iTJSDispatch2,
) -> tjs_error {
    log!("[simple-class]ncm_from");
    let data = ttstr::from("return new SimpleClass();");
    unsafe {
        TVPExecuteScript(&data, result);
    }
    0
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
    let fname = ttstr::from("from");
    let fname = fname.c_str();
    unsafe {
        TJSNativeClassRegisterNCM(
            classobj,
            fname,
            TJSCreateNativeClassConstructor(Some(ncm_from)) as *mut _,
            name,
            tTJSNativeInstanceType_nitClass,
            TJS_STATICMEMBER,
        );
    }
    classobj as *mut iTJSDispatch2
}

struct NClass {}

#[Tjs2Class]
impl NClass {
    fn new() -> Self {
        log!("NClass: created");
        Self {}
    }

    fn invalidate(&self) {
        log!("NClass: invalidate");
    }
}

impl Drop for NClass {
    fn drop(&mut self) {
        log!("NClass: Dropped");
    }
}

struct MyPoint {
    x: i64,
    y: i64,
}

#[Tjs2Class]
impl MyPoint {
    fn new(x: Option<i64>, y: Option<i64>) -> Self {
        Self {
            x: x.unwrap_or_default(),
            y: y.unwrap_or_default(),
        }
    }

    fn from(s: String) -> anyhow::Result<Self> {
        let mut s = s.split(',');
        let x = s.next().ok_or_else(|| anyhow!("x is missing"))?.parse()?;
        let y = s.next().ok_or_else(|| anyhow!("y is missing"))?.parse()?;
        Ok(Self { x, y })
    }

    fn get_x(&self) -> i64 {
        self.x
    }
    fn get_y(&self) -> i64 {
        self.y
    }
    fn set_x(&mut self, x: i64) {
        self.x = x;
    }
    fn set_y(&mut self, y: i64) {
        self.y = y;
    }
    #[tjs(case = camel)]
    fn to_string(&self) -> String {
        format!("{},{}", self.x, self.y)
    }
    fn xy(x: i64, y: i64) -> String {
        format!("{x},{y}")
    }
}

impl Drop for MyPoint {
    fn drop(&mut self) {
        log!("MyPoint: Dropped x={} y={}", self.x, self.y);
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let simple_class = create_native_class();
    let n_class = NClass::create_native_class().1;
    let my_point = MyPoint::create_native_class().1;
    register_var!(simple_class, n_class, my_point);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[simple-class]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(simple_class, n_class, my_point);
    log!("[simple-class]unlinked plugin");
    unsafe { TVPUninitImportStub() };
    0
}
