#![cfg_attr(any(docsrs, feature = "unstable"), feature(doc_cfg))]
#[cfg(feature = "com")]
pub mod com;
#[allow(non_snake_case, non_camel_case_types)]
pub mod tp_stub;

use tp_stub::*;
#[cfg(feature = "com")]
use windows::Win32::System::Com;
#[cfg(feature = "com")]
use windows::core::Interface;

/// Log message to TVP
///
/// Line break is not needed at the end of message.
pub fn log(msg: &str) {
    let s = msg.into();
    unsafe { TVPAddLog(&s) };
}

/// `print` like macro to log message to TVP
///
/// Line break is not needed at the end of message.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        log(&format!($($arg)*));
    };
}

/// Throw exception message to TVP
///
/// This function never returns
pub fn throw_exception_message(msg: &str) -> ! {
    let mut data: Vec<_> = msg.encode_utf16().collect();
    data.push(0);
    unsafe {
        TVPThrowExceptionMessage(data.as_ptr());
        // TVPThrowExceptionMessage never returns
        std::hint::unreachable_unchecked()
    }
}

/// `print` like macro to throw exception message to TVP
///
/// This macro never returns
#[macro_export]
macro_rules! throw_exception_message {
    ($($arg:tt)*) => {
        throw_exception_message(&format!($($arg)*));
    };
}

/// Throw null access error to TVP
pub fn throw_null_access() -> ! {
    unsafe {
        TJSThrowNullAccess();
        // TJSThrowNullAccess never returns
        std::hint::unreachable_unchecked()
    }
}

#[cfg(feature = "com")]
/// Create a [`IStream`](Com::IStream) from name.
///
/// `flags` - TJS_BS_* flags
///
/// ```no_run
/// use krkrz_plugin_base::{com::*, tp_stub::*, *};
/// use std::io::Read;
/// if let Some(stream) = create_istream("test.txt", TJS_BS_READ) {
///     let mut stream = IStreamWrapper(stream);
///     let mut text = String::new();
///     stream.read_to_string(&mut text);
/// }
/// ```
pub fn create_istream(name: &str, flags: u32) -> Option<Com::IStream> {
    let name = ttstr::from(name);
    let stream = unsafe { TVPCreateIStream(&name, flags) };
    if stream.is_null() {
        None
    } else {
        Some(unsafe { Com::IStream::from_raw(stream as *mut _) })
    }
}
