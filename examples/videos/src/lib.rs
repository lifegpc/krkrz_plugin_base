//! WebM/VP9/Opus DirectShow plugin for krkrz.

#![cfg_attr(not(windows), allow(dead_code))]

#[macro_export]
macro_rules! videos_log {
    ($($arg:tt)*) => {{
        #[cfg(feature = "log")]
        {
            krkrz_plugin_base::log!($($arg)*);
        }
    }};
}

use krkrz_plugin_base::tp_stub;

#[cfg(feature = "webm")]
pub mod webm;

#[cfg(feature = "vp9")]
pub mod vp9;

#[cfg(feature = "opus")]
pub mod opus;

#[cfg(windows)]
mod directshow;

/// Features compiled into this plugin build.
pub mod features {
    pub const WEBM: bool = cfg!(feature = "webm");
    pub const VP9: bool = cfg!(feature = "vp9");
    pub const OPUS: bool = cfg!(feature = "opus");
}

#[cfg(windows)]
static mut REF_COUNT_AT_LINK: tp_stub::tjs_int = 0;

#[cfg(all(windows, feature = "webm", feature = "vp9"))]
static mut WEBM_FORMAT_GUID: windows::core::GUID =
    windows::core::GUID::from_u128(0x6ed33d7a_2b16_4f9d_8f13_fbd3cf8c32ff);

#[cfg(windows)]
#[unsafe(export_name = "V2Link")]
unsafe extern "system-unwind" fn v2_link(exporter: *mut tp_stub::iTVPFunctionExporter) -> i32 {
    unsafe {
        tp_stub::TVPInitImportStub(exporter);
    }
    crate::videos_log!("[videos] V2Link begin");

    // The graph requires video; audio may be omitted by the Opus feature.
    #[cfg(all(feature = "webm", feature = "vp9"))]
    unsafe {
        let extension = tp_stub::ttstr::from(".webm");
        crate::videos_log!("[videos] registering .webm DirectShow filters");
        tp_stub::TVPRegisterDSVideoCodec(
            &extension,
            (&raw mut WEBM_FORMAT_GUID).cast(),
            Some(directshow::splitter_factory),
            Some(directshow::vp9_factory),
            Some(directshow::opus_factory),
            std::ptr::null_mut(),
        );
    }
    unsafe {
        REF_COUNT_AT_LINK = tp_stub::TVPPluginGlobalRefCount;
    }
    crate::videos_log!("[videos] V2Link complete");
    0
}

#[cfg(windows)]
#[unsafe(export_name = "V2Unlink")]
unsafe extern "system-unwind" fn v2_unlink() -> i32 {
    crate::videos_log!("[videos] V2Unlink begin");
    if unsafe { tp_stub::TVPPluginGlobalRefCount > REF_COUNT_AT_LINK } {
        crate::videos_log!("[videos] Cannot unlink while plugin references are active");
        return tp_stub::TJS_E_FAIL;
    }

    #[cfg(all(feature = "webm", feature = "vp9"))]
    unsafe {
        let extension = tp_stub::ttstr::from(".webm");
        tp_stub::TVPUnregisterDSVideoCodec(
            &extension,
            (&raw mut WEBM_FORMAT_GUID).cast(),
            Some(directshow::splitter_factory),
            Some(directshow::vp9_factory),
            Some(directshow::opus_factory),
            std::ptr::null_mut(),
        );
    }
    unsafe {
        tp_stub::TVPUninitImportStub();
    }
    0
}
