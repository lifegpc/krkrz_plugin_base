#[allow(non_snake_case)]
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
