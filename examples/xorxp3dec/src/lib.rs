use krkrz_plugin_base::{tp_stub::*, *};

const KEY: u8 = match u8::from_str_radix(
    match option_env!("KEY") {
        Some(key) => key,
        None => "173",
    },
    10,
) {
    Ok(n) => n,
    Err(_) => panic!("KEY cannot be parsed as u8."),
};

unsafe extern "system" fn filter(info: *mut tTVPXP3ExtractionFilterInfo) {
    if info.is_null() {
        return;
    }
    let info = unsafe { &mut *info };
    const EXPECTED: usize = size_of::<tTVPXP3ExtractionFilterInfo>();
    if info.SizeOfSelf as usize != EXPECTED {
        throw_exception_message!(
            "Incompatible tTVPXP3ExtractionFilterInfo size, actual = {}, expected = {EXPECTED}",
            info.SizeOfSelf
        );
    }
    // Skip if filehash is 0
    if info.FileHash == 0 || info.Buffer.is_null() {
        return;
    }
    let buf =
        unsafe { std::slice::from_raw_parts_mut(info.Buffer as *mut u8, info.BufferSize as usize) };
    for b in buf.iter_mut() {
        *b ^= KEY;
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe {
        TVPInitImportStub(exporter);
        TVPSetXP3ArchiveExtractionFilter(Some(filter));
    }
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    unsafe {
        TVPSetXP3ArchiveExtractionFilter(None);
        TVPUninitImportStub();
    }
    0
}
