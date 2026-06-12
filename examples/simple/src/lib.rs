use krkrz_plugin_base::{*, tp_stub::*};

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let s = "[simple plugin]hello world".into();
    unsafe { TVPAddLog(&s) };
    let num = 2333;
    log!("[simple plugin] Log with macro: {num}");
    let mut s = "[simple plugin] AddSign ".into();
    unsafe { TVPAddLog(&s) };
    s += "More";
    unsafe { TVPAddLog(&s) };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    let s = "[simple plugin]unlinked".into();
    unsafe { TVPAddLog(&s) };
    0
}
