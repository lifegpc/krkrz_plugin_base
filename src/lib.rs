#![cfg_attr(any(docsrs, feature = "unstable"), feature(doc_cfg))]
pub mod args;
pub mod r#async;
#[cfg(feature = "com")]
pub mod com;
#[cfg(feature = "serde")]
pub mod de;
pub mod param;
#[cfg(feature = "serde")]
pub mod ser;
#[allow(non_snake_case, non_camel_case_types)]
pub mod tp_stub;
pub mod types;
#[cfg(feature = "serde")]
pub mod value;

use tp_stub::*;
#[cfg(feature = "com")]
use windows::Win32::System::Com;
#[cfg(feature = "com")]
use windows_core::Interface;

pub use args::{Tjs2ArgsArray, Tjs2ArgsVec};
pub use r#async::AsyncExecutor;
#[cfg(feature = "serde")]
pub use krkrz_plugin_base_macros::tjs2_serde_get_args;
pub use krkrz_plugin_base_macros::{
    Tjs2Class, generate_origin_static_block, register_var, tjs, tjs_w, tjs2_function,
    tjs2_get_args, unregister_var,
};
pub use param::TjsParam;
#[cfg(feature = "serde")]
pub use value::TJSVariant;

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
        $crate::log(&format!($($arg)*));
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
        $crate::throw_exception_message(&format!($($arg)*));
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

/// Reference-counted smart pointer to the underlying Rust instance of a [`Tjs2Class`](trait@Tjs2Class).
///
/// Automatically calls `add_ref()` on the backing [`iTJSDispatch2`] object upon creation
/// and `release()` when dropped.
pub struct TjsClassRef<T> {
    obj: *mut iTJSDispatch2,
    inner: *mut T,
}

impl<T> TjsClassRef<T> {
    /// Creates a new `TjsClassRef` and increments the reference count of `obj`.
    ///
    /// # Safety
    /// `obj` and `inner` must be valid non-null pointers, and `inner` must remain valid
    /// for as long as `obj` has a positive reference count.
    pub unsafe fn new(obj: *mut iTJSDispatch2, inner: *mut T) -> Self {
        unsafe {
            (*obj).add_ref();
        }
        Self { obj, inner }
    }
}

impl<T> std::ops::Deref for TjsClassRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl<T> std::ops::DerefMut for TjsClassRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

impl<T> AsRef<T> for TjsClassRef<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for TjsClassRef<T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T> Drop for TjsClassRef<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.obj).release();
        }
    }
}

/// Trait implemented for Rust types annotated with [`#[Tjs2Class]`](macro@crate::Tjs2Class).
pub trait Tjs2Class: Sized {
    /// Registers the native class with TJS2 and returns `(class_id, class_dispatch_ptr)`.
    fn create_native_class() -> (i32, *mut iTJSDispatch2);

    /// Creates a new TJS instance of this class initialized with the given Rust value `obj`.
    ///
    /// The native class must already be registered in the TJS global scope (matching the
    /// class's `new` or `class_name` attribute). Returns an owned `*mut iTJSDispatch2`
    /// with reference count incremented.
    fn create(obj: Self) -> *mut iTJSDispatch2;

    /// Returns `true` if `obj` is a non-null `iTJSDispatch2` carrying a native instance
    /// of this class (checked via its registered `classid`).
    fn is_instance_of(obj: *mut iTJSDispatch2) -> bool;

    /// Returns `true` if `obj` is an object `tTJSVariant` carrying a native instance
    /// of this class, or `false` if `obj` is not an object or belongs to another class.
    fn is_instance_of2(obj: &tTJSVariant) -> bool;

    /// Borrows the underlying Rust instance from `obj`.
    ///
    /// The caller does not need to call `add_ref()` before calling this method; `get_ref`
    /// increments the reference count of `obj` internally and decrements it when the
    /// returned smart pointer is dropped.
    ///
    /// # Panics
    /// Panics if `obj` is null, the native class has not been registered, `obj` is not
    /// an instance of this class, or its native data is uninitialized / invalidated (`None`).
    fn get_ref(obj: *mut iTJSDispatch2)
    -> impl std::ops::Deref<Target = Self> + std::ops::DerefMut;

    /// Borrows the underlying Rust instance from a [`tTJSVariant`].
    ///
    /// The caller does not need to call `add_ref()` before calling this method; `get_ref2`
    /// increments the reference count of the underlying `iTJSDispatch2` internally and
    /// decrements it when the returned smart pointer is dropped.
    ///
    /// # Panics
    /// Panics if `obj` is not an object variant, the native class has not been registered,
    /// `obj` is not an instance of this class, or its native data is uninitialized /
    /// invalidated (`None`).
    fn get_ref2(obj: &mut tTJSVariant) -> impl std::ops::Deref<Target = Self> + std::ops::DerefMut;
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests only type-check macro expansions. They are deliberately not
    // executed because creating a TJS variant requires an initialized engine.
    #[allow(dead_code)]
    fn get_args_macro_typechecks() {
        let mut fixed = tjs2_get_args!(12, "text");
        let _ = (fixed.len(), fixed.as_mut_ptr());

        let values = vec![34_i64, 56_i64];
        let mut dynamic = tjs2_get_args!(12, *values, "text");
        let _ = (dynamic.len(), dynamic.as_mut_ptr());
    }

    #[cfg(feature = "serde")]
    #[allow(dead_code)]
    fn serde_get_args_macro_typechecks() -> Result<(), ser::SerError> {
        let mut fixed = tjs2_serde_get_args!(12, "text")?;
        let _ = (fixed.len(), fixed.as_mut_ptr());

        let values = vec![34_i64, 56_i64];
        let mut dynamic = tjs2_serde_get_args!(12, *values, "text")?;
        let _ = (dynamic.len(), dynamic.as_mut_ptr());
        Ok(())
    }
}
