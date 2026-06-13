//! Low level api for tp_stub
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

    pub fn alloc_buffer(&mut self, len: tjs_uint) -> *mut tjs_char {
        type Type = extern "system" fn(s: *mut tTJSString, len: tjs_uint) -> *mut tjs_char;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr314573cca30a7c2aecc9166fbf5400c9,
                "tjs_char * tTJSString::AllocBuffer(tjs_uint)\0",
                Type
            )
        };
        ptr(self, len)
    }

    pub fn append_buffer(&mut self, len: tjs_uint) -> *mut tjs_char {
        type Type = extern "system" fn(s: *mut tTJSString, len: tjs_uint) -> *mut tjs_char;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr03da356426c038fad663c836c3e330ef,
                "tjs_char * tTJSString::AppendBuffer(tjs_uint)\0",
                Type
            )
        };
        ptr(self, len)
    }

    pub fn fix_len(&mut self) {
        type Type = extern "system" fn(s: *mut tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr31dbebdedc08d75e34a2cd564ce60586,
                "void tTJSString::FixLen()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn replace(&mut self, from: &tTJSString, to: &tTJSString, forall: bool) {
        type Type = extern "system" fn(
            s: *mut tTJSString,
            from: *const tTJSString,
            to: *const tTJSString,
            forall: bool,
        );
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrd9224ad7a0de743a7eea15fdb2c5f934,
                "void tTJSString::Replace(const tTJSString &,const tTJSString &,bool)\0",
                Type
            )
        };
        ptr(self, from as *const _, to as *const _, forall)
    }

    pub fn to_lower_case(&mut self) {
        type Type = extern "system" fn(s: *mut tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrc01b0720b49ce4f792446d8965d2c31f,
                "void tTJSString::ToLowerCase()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn to_upper_case(&mut self) {
        type Type = extern "system" fn(s: *mut tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr4af47e46a11e1357cb994f405289d13e,
                "void tTJSString::ToUppserCase()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn get_hint(&mut self) -> *mut tjs_uint32 {
        type Type = extern "system" fn(s: *mut tTJSString) -> *mut tjs_uint32;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr25b6dafa19bfa5bde1a8b519da248f82,
                "tjs_uint32 * tTJSString::GetHint()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn independ(&mut self) -> *mut tjs_char {
        type Type = extern "system" fn(s: *mut tTJSString) -> *mut tjs_char;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr72425405819c900aec719491cbd90c6d,
                "tjs_char * tTJSString::Independ()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn c_str(&self) -> *const tjs_char {
        type Type = extern "system" fn(s: *const tTJSString) -> *const tjs_char;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtra79942af73f33bff6e432c9fd808e469,
                "const tjs_char * tTJSString::c_str() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_variant_string_no_add_ref(&self) -> *mut tTJSVariantString {
        type Type = extern "system" fn(s: *const tTJSString) -> *mut tTJSVariantString;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrdf106470a4141ebc7eda22160859ffdc,
                "tTJSVariantString * tTJSString::AsVariantStringNoAddRef() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_integer(&self) -> tjs_int64 {
        type Type = extern "system" fn(s: *const tTJSString) -> tjs_int64;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr469bc225b0ecd9561aae5a46b85ded42,
                "tjs_int64 tTJSString::AsInteger() const\0",
                Type
            )
        };
        ptr(self)
    }

    #[inline(always)]
    pub fn compare_ic<T: ?Sized>(&self, rhs: &T) -> tjs_int
    where
        Self: tTJSStringCompareIC<T>,
    {
        self.compare(rhs)
    }

    #[inline(always)]
    pub fn equal_ignore_case<T: ?Sized>(&self, rhs: &T) -> bool
    where
        Self: tTJSStringCompareIC<T>,
    {
        self.compare(rhs) == 0
    }

    pub fn get(&self, i: tjs_uint) -> tjs_char {
        type Type = extern "system" fn(s: *const tTJSString, tjs_uint) -> tjs_char;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr564b37278b50f4e5597dff6540868d49,
                "tjs_char tTJSString::operator [](tjs_uint) const\0",
                Type
            )
        };
        ptr(self, i)
    }

    pub fn as_lower_case(&self, dest: &mut tTJSString) {
        type Type = extern "system" fn(s: *const tTJSString, *mut tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr890b3a4831b824653e919b4a5197358d,
                "void tTJSString::AsLowerCase(tTJSString &) const\0",
                Type
            )
        };
        ptr(self, dest)
    }

    pub fn as_upper_case(&self, dest: &mut tTJSString) {
        type Type = extern "system" fn(s: *const tTJSString, *mut tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr2dfa6c77c5051d160b8a06f540e0d68b,
                "void tTJSString::AsUpperCase(tTJSString &) const\0",
                Type
            )
        };
        ptr(self, dest)
    }

    pub fn escape_c(&self, dest: &mut tTJSString) {
        type Type = extern "system" fn(s: *const tTJSString, *mut tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr05f88567d510fd84659ccbf493f647ed,
                "void tTJSString::EscapeC(tTJSString &) const\0",
                Type
            )
        };
        ptr(self, dest)
    }

    pub fn unescape_c(&self, dest: &mut tTJSString) {
        type Type = extern "system" fn(s: *const tTJSString, *mut tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr7166b8f7bb9688c980e4fa172f06f30c,
                "void tTJSString::UnescapeC(tTJSString &) const\0",
                Type
            )
        };
        ptr(self, dest)
    }

    #[inline(always)]
    pub fn starts_with<T: ?Sized>(&self, string: &T) -> bool
    where
        Self: tTJSStringStartsWith<T>,
    {
        tTJSStringStartsWith::starts_with(self, string)
    }

    pub fn get_narrow_str_len(&self) -> tjs_int {
        type Type = extern "system" fn(s: *const tTJSString) -> tjs_int;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtra57696ca0c157cd7d3cd4e58c1df957c,
                "tjs_int tTJSString::GetNarrowStrLen() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn to_narrow_str(&self, dest: *mut tjs_nchar, destmaxlen: tjs_int) {
        type Type = extern "system" fn(s: *const tTJSString, *mut tjs_nchar, tjs_int);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr1aea9f8a38bbb875b6d052f330da9178,
                "void tTJSString::ToNarrowStr(tjs_nchar *,tjs_int) const\0",
                Type
            )
        };
        ptr(self, dest, destmaxlen)
    }

    pub fn is_empty(&self) -> bool {
        type Type = extern "system" fn(s: *const tTJSString) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr2d3b3d6e22ee139cda9eee47dc031945,
                "bool tTJSString::IsEmpty() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn get_len(&self) -> tjs_int {
        type Type = extern "system" fn(s: *const tTJSString) -> tjs_int;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr8ff49e56c3c4c566561dcdd5c9ecc4db,
                "tjs_int tTJSString::GetLen() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn length(&self) -> tjs_int {
        type Type = extern "system" fn(s: *const tTJSString) -> tjs_int;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr490b547e93e40082d0b83312467104f9,
                "tjs_int tTJSString::length() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn get_last_char(&self) -> tjs_char {
        type Type = extern "system" fn(s: *const tTJSString) -> tjs_char;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr2c1ef06748df47df52b586ac0fbc6a34,
                "tjs_char tTJSString::GetLastChar() const\0",
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

impl PartialEq<tTJSString> for tTJSString {
    fn eq(&self, other: &tTJSString) -> bool {
        type Type = extern "system" fn(s: *const tTJSString, *const tTJSString) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtra6663c078b3aa79b39ee2d09f3875765,
                "bool tTJSString::operator ==(const tTJSString &) const\0",
                Type
            )
        };
        ptr(self, other)
    }

    fn ne(&self, other: &tTJSString) -> bool {
        type Type = extern "system" fn(s: *const tTJSString, *const tTJSString) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrefbe634ce4f13633e220cae167cf63fb,
                "bool tTJSString::operator !=(const tTJSString &) const\0",
                Type
            )
        };
        ptr(self, other)
    }
}

pub trait tTJSStringCompareIC<Rhs: ?Sized> {
    fn compare(&self, target: &Rhs) -> tjs_int;
}

impl tTJSStringCompareIC<tTJSString> for tTJSString {
    fn compare(&self, target: &tTJSString) -> tjs_int {
        type Type = extern "system" fn(s: *const tTJSString, *const tTJSString) -> tjs_int;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr57f4147bcc09e4e4442ffc9b0895727e,
                "tjs_int tTJSString::CompareIC(const tTJSString &) const\0",
                Type
            )
        };
        ptr(self, target)
    }
}

impl PartialEq<*const tjs_char> for tTJSString {
    fn eq(&self, other: &*const tjs_char) -> bool {
        type Type = extern "system" fn(s: *const tTJSString, *const tjs_char) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr1fb2d2e44cf83aebef7b26fd6b20bc2b,
                "bool tTJSString::operator ==(const tjs_char *) const\0",
                Type
            )
        };
        ptr(self, *other)
    }

    fn ne(&self, other: &*const tjs_char) -> bool {
        type Type = extern "system" fn(s: *const tTJSString, *const tjs_char) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrbd6aa777bac947f5cffd891e9c724794,
                "bool tTJSString::operator !=(const tjs_char *) const\0",
                Type
            )
        };
        ptr(self, *other)
    }
}

impl tTJSStringCompareIC<*const tjs_char> for tTJSString {
    fn compare(&self, target: &*const tjs_char) -> tjs_int {
        type Type = extern "system" fn(s: *const tTJSString, *const tjs_char) -> tjs_int;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr83c662330b75d616cdc8a4e11d7ababa,
                "tjs_int tTJSString::CompareIC(const tjs_char *) const\0",
                Type
            )
        };
        ptr(self, *target)
    }
}

impl PartialEq<[tjs_char]> for tTJSString {
    fn eq(&self, other: &[tjs_char]) -> bool {
        self.eq(&other.as_ptr())
    }
    fn ne(&self, other: &[tjs_char]) -> bool {
        self.ne(&other.as_ptr())
    }
}

impl tTJSStringCompareIC<[tjs_char]> for tTJSString {
    fn compare(&self, target: &[tjs_char]) -> tjs_int {
        self.compare(&target.as_ptr())
    }
}

impl PartialEq<str> for tTJSString {
    fn eq(&self, other: &str) -> bool {
        let ty: Self = other.into();
        self.eq(&ty)
    }
}

impl tTJSStringCompareIC<str> for tTJSString {
    fn compare(&self, target: &str) -> tjs_int {
        let ty: Self = target.into();
        self.compare(&ty)
    }
}

impl PartialOrd<tTJSString> for tTJSString {
    fn partial_cmp(&self, other: &tTJSString) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        if self == other {
            Some(Equal)
        } else if self.lt(other) {
            Some(Less)
        } else if self.gt(other) {
            Some(Greater)
        } else {
            None
        }
    }

    fn lt(&self, other: &tTJSString) -> bool {
        type Type = extern "system" fn(s: *const tTJSString, *const tTJSString) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrbbde02fe30c8a6cadb7073174ea3a874,
                "bool tTJSString::operator <(const tTJSString &) const\0",
                Type
            )
        };
        ptr(self, other)
    }

    fn gt(&self, other: &tTJSString) -> bool {
        type Type = extern "system" fn(s: *const tTJSString, *const tTJSString) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrcc1c14f63867f90bc883de03e9212cbc,
                "bool tTJSString::operator >(const tTJSString &) const\0",
                Type
            )
        };
        ptr(self, other)
    }
}

impl Add<&tTJSString> for &tTJSString {
    type Output = tTJSString;
    fn add(self, rhs: &tTJSString) -> Self::Output {
        type Type = extern "system" fn(s: *const tTJSString, *const tTJSString) -> tTJSString;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr236e007b32bc2631b5f6dc1eda6be0a9,
                "tTJSString tTJSString::operator +(const tTJSString &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Add<&tTJSString> for tTJSString {
    type Output = tTJSString;
    fn add(self, rhs: &tTJSString) -> Self::Output {
        &self + rhs
    }
}

impl Add<*const tjs_char> for &tTJSString {
    type Output = tTJSString;
    fn add(self, rhs: *const tjs_char) -> Self::Output {
        type Type = extern "system" fn(s: *const tTJSString, *const tjs_char) -> tTJSString;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrcfbb9809e0e6d954b2652856e935ced9,
                "tTJSString tTJSString::operator +(const tjs_char *) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Add<*const tjs_char> for tTJSString {
    type Output = tTJSString;
    fn add(self, rhs: *const tjs_char) -> Self::Output {
        (&self).add(rhs)
    }
}

impl Add<&[tjs_char]> for &tTJSString {
    type Output = tTJSString;
    fn add(self, rhs: &[tjs_char]) -> Self::Output {
        self + rhs.as_ptr()
    }
}

impl Add<&[tjs_char]> for tTJSString {
    type Output = tTJSString;
    fn add(self, rhs: &[tjs_char]) -> Self::Output {
        &self + rhs.as_ptr()
    }
}

impl Add<&str> for &tTJSString {
    type Output = tTJSString;
    fn add(self, rhs: &str) -> Self::Output {
        let ty: tTJSString = rhs.into();
        self + &ty
    }
}

impl Add<&str> for tTJSString {
    type Output = tTJSString;
    fn add(self, rhs: &str) -> Self::Output {
        &self + rhs
    }
}

impl Add<tjs_char> for &tTJSString {
    type Output = tTJSString;
    fn add(self, rhs: tjs_char) -> Self::Output {
        type Type = extern "system" fn(s: *const tTJSString, tjs_char) -> tTJSString;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr60ee96ae4a7704340bef20fb35ba6ade,
                "tTJSString tTJSString::operator +(tjs_char) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Add<tjs_char> for tTJSString {
    type Output = tTJSString;
    fn add(self, rhs: tjs_char) -> Self::Output {
        (&self).add(rhs)
    }
}

pub trait tTJSStringStartsWith<T: ?Sized> {
    fn starts_with(&self, string: &T) -> bool;
}

impl tTJSStringStartsWith<tTJSString> for tTJSString {
    fn starts_with(&self, string: &tTJSString) -> bool {
        type Type = extern "system" fn(s: *const tTJSString, *const tTJSString) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrdd44464bd8430a5be5fef0cffcd97117,
                "bool tTJSString::StartsWith(const tTJSString &) const\0",
                Type
            )
        };
        ptr(self, string)
    }
}

impl tTJSStringStartsWith<*const tjs_char> for tTJSString {
    fn starts_with(&self, string: &*const tjs_char) -> bool {
        type Type = extern "system" fn(s: *const tTJSString, *const tjs_char) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrb9456ecba8b7898d80d2e5caa64035c9,
                "bool tTJSString::StartsWith(const tjs_char *) const\0",
                Type
            )
        };
        ptr(self, *string)
    }
}

impl tTJSStringStartsWith<[tjs_char]> for tTJSString {
    fn starts_with(&self, string: &[tjs_char]) -> bool {
        self.starts_with(&string.as_ptr())
    }
}

impl tTJSStringStartsWith<str> for tTJSString {
    fn starts_with(&self, string: &str) -> bool {
        let ty: Self = string.into();
        self.starts_with(&ty)
    }
}
