#[allow(non_snake_case, non_camel_case_types)]
pub mod tp_stub;

use tp_stub::*;

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
