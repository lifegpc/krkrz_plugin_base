#[allow(warnings)]
mod generated;

pub use generated::*;
use std::ffi::*;
use std::ops::*;
use std::ptr;

#[inline(always)]
pub fn TJS_FAILED(hr: tjs_error) -> bool {
    hr < 0
}

#[inline(always)]
pub fn TJS_SUCCEEDED(hr: tjs_error) -> bool {
    hr >= 0
}

impl From<tjs_int> for tTJSStringBufferLength {
    fn from(value: tjs_int) -> Self {
        Self { n: value }
    }
}

#[test]
fn test_inline() {
    assert!(unsafe { TJSIsObjectValid(TJS_S_TRUE) });
}

macro_rules! import_func {
    ($ptr:expr, $name:expr, $t:ty) => {{
        // check name at compile time
        const _: () = {
            const NAME: &str = $name;
            const BYTES: &[u8] = unsafe { std::mem::transmute(NAME) };
            assert!(
                BYTES.len() > 0 && BYTES[BYTES.len() - 1] == b'\0',
                "name must ends with \\0"
            );
        };
        if $ptr.is_null() {
            let name_ptr = $name.as_ptr() as *const i8;
            $ptr = TVPGetImportFuncPtr(name_ptr);
        }
        std::mem::transmute::<*mut c_void, $t>($ptr)
    }};
}

impl tTJSString {
    pub fn new() -> Self {
        type Type = extern "system" fn(s: *mut tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrd83a866389246d824efcc83303a04484,
                "tTJSString::tTJSString()\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my);
        my
    }

    pub fn clear(&mut self) {
        type Type = extern "system" fn(s: *mut tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr168cf4c1b9ef70b98f2e0ab3695a4f3b,
                "void tTJSString::Clear()\0",
                Type
            )
        };
        ptr(self)
    }
}

impl Clone for tTJSString {
    fn clone(&self) -> Self {
        Self::from(self)
    }
}

impl From<&tTJSString> for tTJSString {
    fn from(value: &tTJSString) -> Self {
        type Type = extern "system" fn(s: *mut tTJSString, rhs: *const tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr6cf6f332a6a14a15e8dce62301f5c840,
                "tTJSString::tTJSString(const tTJSString &)\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my, value as *const _);
        my
    }
}

impl From<*mut tTJSVariantString> for tTJSString {
    fn from(value: *mut tTJSVariantString) -> Self {
        type Type = extern "system" fn(s: *mut tTJSString, rhs: *mut tTJSVariantString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr566eeea3c5f009b0fc6fa123ba30f496,
                "tTJSString::tTJSString(tTJSVariantString *)\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my, value);
        my
    }
}

impl From<&mut tTJSVariantString> for tTJSString {
    fn from(value: &mut tTJSVariantString) -> Self {
        Self::from(value as *mut _)
    }
}

impl From<*const tjs_char> for tTJSString {
    fn from(value: *const tjs_char) -> Self {
        type Type = extern "system" fn(s: *mut tTJSString, str: *const tjs_char);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr88806e38e35c73b36acadd4061a4fe0b,
                "tTJSString::tTJSString(const tjs_char *)\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my, value);
        my
    }
}

impl From<&[tjs_char]> for tTJSString {
    fn from(value: &[tjs_char]) -> Self {
        Self::from(value.as_ptr())
    }
}

impl From<&str> for tTJSString {
    /// Convert Rust string to [tTJSString]. A null byte will append to the end.
    fn from(value: &str) -> Self {
        let mut encoded: Vec<_> = value.encode_utf16().collect();
        encoded.push(0);
        Self::from(encoded.as_ptr())
    }
}

impl From<*const tjs_nchar> for tTJSString {
    fn from(value: *const tjs_nchar) -> Self {
        type Type = extern "system" fn(s: *mut tTJSString, str: *const tjs_nchar);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr3bb69d3886159aaecc333b6ff17287bf,
                "tTJSString::tTJSString(const tjs_nchar *)\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my, value);
        my
    }
}

impl From<&[tjs_nchar]> for tTJSString {
    fn from(value: &[tjs_nchar]) -> Self {
        Self::from(value.as_ptr())
    }
}

impl From<tTJSStringBufferLength> for tTJSString {
    fn from(value: tTJSStringBufferLength) -> Self {
        type Type = extern "system" fn(s: *mut tTJSString, len: tTJSStringBufferLength);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr3e36278551a9c8b29cb2e8017db6af0d,
                "tTJSString::tTJSString(const tTJSStringBufferLength)\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my, value);
        my
    }
}

impl From<tjs_char> for tTJSString {
    fn from(value: tjs_char) -> Self {
        type Type = extern "system" fn(s: *mut tTJSString, rch: tjs_char);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr5de99d84f3dc902cb0812fb85a7d5c88,
                "tTJSString::tTJSString(tjs_char)\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my, value);
        my
    }
}

impl From<&tTJSVariant> for tTJSString {
    fn from(value: &tTJSVariant) -> Self {
        type Type = extern "system" fn(s: *mut tTJSString, val: *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr31e85cbc73f8fbd4cea895a751480059,
                "tTJSString::tTJSString(const tTJSVariant &)\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my, value as *const _);
        my
    }
}

impl From<(&tTJSString, c_int)> for tTJSString {
    fn from(value: (&tTJSString, c_int)) -> Self {
        type Type = extern "system" fn(s: *mut tTJSString, str: *const tTJSString, n: c_int);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr6ae29e405ede762f1a89a9dd526cb36e,
                "tTJSString::tTJSString(const tTJSString &,int)\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my, value.0 as *const _, value.1);
        my
    }
}

impl From<(*const tjs_char, c_int)> for tTJSString {
    fn from(value: (*const tjs_char, c_int)) -> Self {
        type Type = extern "system" fn(s: *mut tTJSString, str: *const tjs_char, n: c_int);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrc95bd66d95c153cdac41b5243e555f5f,
                "tTJSString::tTJSString(const tjs_char *,int)\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my, value.0, value.1);
        my
    }
}

impl From<(&[tjs_char], c_int)> for tTJSString {
    fn from(value: (&[tjs_char], c_int)) -> Self {
        Self::from((value.0.as_ptr(), value.1))
    }
}

impl From<tjs_int> for tTJSString {
    fn from(value: tjs_int) -> Self {
        type Type = extern "system" fn(s: *mut tTJSString, n: tjs_int);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr72a67e9c52fd27dbb66eded47efeea74,
                "tTJSString::tTJSString(tjs_int)\0",
                Type
            )
        };
        let mut my = Self {
            _base: tTJSString_S {
                Ptr: ptr::null_mut(),
            },
        };
        ptr(&mut my, value);
        my
    }
}

impl Drop for tTJSString {
    fn drop(&mut self) {
        type Type = extern "system" fn(s: *mut tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrfb13e41bda53e4e59403e3e14effccd6,
                "tTJSString::~ tTJSString()\0",
                Type
            )
        };
        ptr(self)
    }
}

impl AddAssign<&tTJSString> for tTJSString {
    fn add_assign(&mut self, rhs: &tTJSString) {
        type Type = extern "system" fn(s: *mut tTJSString, *const tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrcd50da721dfb63f36c1ebb1226830428,
                "void tTJSString::operator +=(const tTJSString &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl AddAssign<*const tTJSVariantString> for tTJSString {
    fn add_assign(&mut self, rhs: *const tTJSVariantString) {
        type Type = extern "system" fn(s: *mut tTJSString, *const tTJSVariantString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrfbba3dd6a087599d1277ae58f6cec18e,
                "void tTJSString::operator +=(const tTJSVariantString *)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl AddAssign<&tTJSVariantString> for tTJSString {
    fn add_assign(&mut self, rhs: &tTJSVariantString) {
        *self += rhs as *const _
    }
}

impl AddAssign<*const tjs_char> for tTJSString {
    fn add_assign(&mut self, rhs: *const tjs_char) {
        type Type = extern "system" fn(s: *mut tTJSString, *const tjs_char);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr43cc5b5a61a6090af83333d115b5b868,
                "void tTJSString::operator +=(const tjs_char *)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl AddAssign<&[tjs_char]> for tTJSString {
    fn add_assign(&mut self, rhs: &[tjs_char]) {
        *self += rhs.as_ptr()
    }
}

impl AddAssign<&str> for tTJSString {
    fn add_assign(&mut self, rhs: &str) {
        *self += &Self::from(rhs)
    }
}

impl AddAssign<tjs_char> for tTJSString {
    fn add_assign(&mut self, rhs: tjs_char) {
        type Type = extern "system" fn(s: *mut tTJSString, tjs_char);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr616fb5060d81eb5bab58647596582df4,
                "void tTJSString::operator +=(tjs_char)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}
