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

    #[inline(always)]
    pub fn assign<T>(&mut self, rhs: T) -> &mut Self
    where
        Self: Assign<T>,
    {
        Assign::assign(self, rhs)
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

pub trait Assign<T: ?Sized> {
    fn assign(&mut self, rhs: T) -> &mut Self;
}

impl Assign<&tTJSString> for tTJSString {
    fn assign(&mut self, rhs: &tTJSString) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSString, *const tTJSString) -> *mut tTJSString;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr18f1ad16c11429707cbf8ea4d1d4a21e,
                "tTJSString & tTJSString::operator =(const tTJSString &)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<*const tjs_char> for tTJSString {
    fn assign(&mut self, rhs: *const tjs_char) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSString, *const tjs_char) -> *mut tTJSString;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr18f1ad16c11429707cbf8ea4d1d4a21e,
                "tTJSString & tTJSString::operator =(const tjs_char *)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<&[tjs_char]> for tTJSString {
    fn assign(&mut self, rhs: &[tjs_char]) -> &mut Self {
        self.assign(rhs.as_ptr())
    }
}

impl Assign<*const tjs_nchar> for tTJSString {
    fn assign(&mut self, rhs: *const tjs_nchar) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSString, *const tjs_nchar) -> *mut tTJSString;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr550f317b573a1256af00586890ae82f1,
                "tTJSString & tTJSString::operator =(const tjs_nchar *)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<&[tjs_nchar]> for tTJSString {
    fn assign(&mut self, rhs: &[tjs_nchar]) -> &mut Self {
        self.assign(rhs.as_ptr())
    }
}

impl Assign<&[u8]> for tTJSString {
    fn assign(&mut self, rhs: &[u8]) -> &mut Self {
        self.assign(rhs.as_ptr() as *const i8)
    }
}

impl Assign<&str> for tTJSString {
    fn assign(&mut self, rhs: &str) -> &mut Self {
        let mut encoded: Vec<_> = rhs.encode_utf16().collect();
        encoded.push(0);
        self.assign(encoded.as_ptr())
    }
}

impl Default for tTJSString {
    fn default() -> Self {
        Self::new()
    }
}

impl iTJSDispatch2 {
    #[inline(always)]
    pub unsafe fn add_ref(&mut self) -> tjs_uint {
        unsafe { ((*self.vtable_).iTJSDispatch2_AddRef)(self) }
    }

    #[inline(always)]
    pub unsafe fn release(&mut self) -> tjs_uint {
        unsafe { ((*self.vtable_).iTJSDispatch2_Release)(self) }
    }

    #[inline(always)]
    pub unsafe fn func_call(
        &mut self,
        flag: tjs_uint32,
        membername: *const tjs_char,
        hint: *mut tjs_uint32,
        result: *mut tTJSVariant,
        numparams: tjs_int,
        param: *mut *mut tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_FuncCall)(
                self, flag, membername, hint, result, numparams, param, objthis,
            )
        }
    }

    #[inline(always)]
    pub unsafe fn func_call_by_num(
        &mut self,
        flag: tjs_uint32,
        num: tjs_int,
        result: *mut tTJSVariant,
        numparams: tjs_int,
        param: *mut *mut tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_FuncCallByNum)(
                self, flag, num, result, numparams, param, objthis,
            )
        }
    }

    #[inline(always)]
    pub unsafe fn prop_get(
        &mut self,
        flag: tjs_uint32,
        membername: *const tjs_char,
        hint: *mut tjs_uint32,
        result: *mut tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_PropGet)(self, flag, membername, hint, result, objthis)
        }
    }

    #[inline(always)]
    pub unsafe fn prop_get_by_num(
        &mut self,
        flag: tjs_uint32,
        num: tjs_int,
        result: *mut tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_PropGetByNum)(self, flag, num, result, objthis) }
    }

    #[inline(always)]
    pub unsafe fn prop_set(
        &mut self,
        flag: tjs_uint32,
        membername: *const tjs_char,
        hint: *mut tjs_uint32,
        param: *const tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_PropSet)(self, flag, membername, hint, param, objthis)
        }
    }

    #[inline(always)]
    pub unsafe fn prop_set_by_num(
        &mut self,
        flag: tjs_uint32,
        num: tjs_int,
        param: *const tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_PropSetByNum)(self, flag, num, param, objthis) }
    }

    #[inline(always)]
    pub unsafe fn get_count(
        &mut self,
        result: *mut tjs_int,
        membername: *const tjs_char,
        hint: *mut tjs_uint32,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_GetCount)(self, result, membername, hint, objthis) }
    }

    #[inline(always)]
    pub unsafe fn get_count_by_num(
        &mut self,
        result: *mut tjs_int,
        num: tjs_int,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_GetCountByNum)(self, result, num, objthis) }
    }

    #[inline(always)]
    pub unsafe fn prop_set_by_vs(
        &mut self,
        flag: tjs_uint32,
        membername: *mut tTJSVariantString,
        param: *const tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_PropSetByVS)(self, flag, membername, param, objthis)
        }
    }

    #[inline(always)]
    pub unsafe fn enum_members(
        &mut self,
        flag: tjs_uint32,
        callback: *mut tTJSVariantClosure,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_EnumMembers)(self, flag, callback, objthis) }
    }

    #[inline(always)]
    pub unsafe fn delete_member(
        &mut self,
        flag: tjs_uint32,
        membername: *const tjs_char,
        hint: *mut tjs_uint32,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_DeleteMember)(self, flag, membername, hint, objthis)
        }
    }

    #[inline(always)]
    pub unsafe fn delete_member_by_num(
        &mut self,
        flag: tjs_uint32,
        num: tjs_int,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_DeleteMemberByNum)(self, flag, num, objthis) }
    }

    #[inline(always)]
    pub unsafe fn invalidate(
        &mut self,
        flag: tjs_uint32,
        membername: *const tjs_char,
        hint: *mut tjs_uint32,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_Invalidate)(self, flag, membername, hint, objthis) }
    }

    #[inline(always)]
    pub unsafe fn invalidate_by_num(
        &mut self,
        flag: tjs_uint32,
        num: tjs_int,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_InvalidateByNum)(self, flag, num, objthis) }
    }

    #[inline(always)]
    pub unsafe fn is_valid(
        &mut self,
        flag: tjs_uint32,
        membername: *const tjs_char,
        hint: *mut tjs_uint32,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_IsValid)(self, flag, membername, hint, objthis) }
    }

    #[inline(always)]
    pub unsafe fn is_valid_by_num(
        &mut self,
        flag: tjs_uint32,
        num: tjs_int,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_IsValidByNum)(self, flag, num, objthis) }
    }

    #[inline(always)]
    pub unsafe fn create_new(
        &mut self,
        flag: tjs_uint32,
        membername: *const tjs_char,
        hint: *mut tjs_uint32,
        result: *mut *mut iTJSDispatch2,
        numparams: tjs_int,
        param: *mut *mut tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_CreateNew)(
                self, flag, membername, hint, result, numparams, param, objthis,
            )
        }
    }

    #[inline(always)]
    pub unsafe fn create_new_by_num(
        &mut self,
        flag: tjs_uint32,
        num: tjs_int,
        result: *mut *mut iTJSDispatch2,
        numparams: tjs_int,
        param: *mut *mut tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_CreateNewByNum)(
                self, flag, num, result, numparams, param, objthis,
            )
        }
    }

    #[inline(always)]
    pub unsafe fn reserved1(&mut self) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_Reserved1)(self) }
    }

    #[inline(always)]
    pub unsafe fn is_instance_of(
        &mut self,
        flag: tjs_uint32,
        membername: *const tjs_char,
        hint: *mut tjs_uint32,
        classname: *const tjs_char,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_IsInstanceOf)(
                self, flag, membername, hint, classname, objthis,
            )
        }
    }

    #[inline(always)]
    pub unsafe fn is_instance_of_by_num(
        &mut self,
        flag: tjs_uint32,
        num: tjs_int,
        classname: *const tjs_char,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_IsInstanceOfByNum)(self, flag, num, classname, objthis)
        }
    }

    #[inline(always)]
    pub unsafe fn operation(
        &mut self,
        flag: tjs_uint32,
        membername: *const tjs_char,
        hint: *mut tjs_uint32,
        result: *mut tTJSVariant,
        param: *const tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_Operation)(
                self, flag, membername, hint, result, param, objthis,
            )
        }
    }

    #[inline(always)]
    pub unsafe fn operation_by_num(
        &mut self,
        flag: tjs_uint32,
        num: tjs_int,
        result: *mut tTJSVariant,
        param: *const tTJSVariant,
        objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_OperationByNum)(self, flag, num, result, param, objthis)
        }
    }

    #[inline(always)]
    pub unsafe fn native_instance_support(
        &mut self,
        flag: tjs_uint32,
        classid: tjs_int32,
        pointer: *mut *mut iTJSNativeInstance,
    ) -> tjs_error {
        unsafe {
            ((*self.vtable_).iTJSDispatch2_NativeInstanceSupport)(self, flag, classid, pointer)
        }
    }

    #[inline(always)]
    pub unsafe fn class_instance_info(
        &mut self,
        flag: tjs_uint32,
        num: tjs_uint,
        value: *mut tTJSVariant,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_ClassInstanceInfo)(self, flag, num, value) }
    }

    #[inline(always)]
    pub unsafe fn reserved2(&mut self) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_Reserved2)(self) }
    }

    #[inline(always)]
    pub unsafe fn reserved3(&mut self) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSDispatch2_Reserved3)(self) }
    }
}

impl iTJSNativeInstance {
    #[inline(always)]
    pub unsafe fn construct(
        &mut self,
        numparams: tjs_int,
        param: *mut *mut tTJSVariant,
        tjs_obj: *mut iTJSDispatch2,
    ) -> tjs_error {
        unsafe { ((*self.vtable_).iTJSNativeInstance_Construct)(self, numparams, param, tjs_obj) }
    }

    #[inline(always)]
    pub unsafe fn invalidate(&mut self) {
        unsafe { ((*self.vtable_).iTJSNativeInstance_Invalidate)(self) }
    }

    #[inline(always)]
    pub unsafe fn destruct(&mut self) {
        unsafe { ((*self.vtable_).iTJSNativeInstance_Destruct)(self) }
    }
}

pub trait TJSNativeInstance {
    fn construct(
        &mut self,
        _numparams: tjs_int,
        _param: *mut *mut tTJSVariant,
        _tjs_obj: *mut iTJSDispatch2,
    ) -> tjs_error {
        TJS_S_OK
    }
    fn invalidate(&mut self) {}
}

#[repr(C)]
pub struct tTJSNativeInstance {
    _base: iTJSNativeInstance,
    ni: Box<dyn TJSNativeInstance>,
}

impl tTJSNativeInstance {
    pub fn new<T: TJSNativeInstance + 'static>(ni: T) -> *mut iTJSNativeInstance {
        static VTABLE: iTJSNativeInstance__bindgen_vtable = iTJSNativeInstance__bindgen_vtable {
            iTJSNativeInstance_Construct: tTJSNativeInstance::construct,
            iTJSNativeInstance_Invalidate: tTJSNativeInstance::invalidate,
            iTJSNativeInstance_Destruct: tTJSNativeInstance::destruct,
        };
        let boxed = Box::new(tTJSNativeInstance {
            _base: iTJSNativeInstance { vtable_: &VTABLE },
            ni: Box::new(ni) as Box<dyn TJSNativeInstance>,
        });
        Box::into_raw(boxed) as *mut iTJSNativeInstance
    }
    unsafe extern "C" fn construct(
        this: *mut iTJSNativeInstance,
        numparams: tjs_int,
        param: *mut *mut tTJSVariant,
        tjs_obj: *mut iTJSDispatch2,
    ) -> tjs_error {
        let self_ = unsafe { &mut *(this as *mut tTJSNativeInstance) };
        self_.ni.construct(numparams, param, tjs_obj)
    }
    unsafe extern "C" fn invalidate(this: *mut iTJSNativeInstance) {
        let self_ = unsafe { &mut *(this as *mut tTJSNativeInstance) };
        self_.ni.invalidate();
    }
    unsafe extern "C" fn destruct(this: *mut iTJSNativeInstance) {
        let _box = unsafe { Box::from_raw(this as *mut tTJSNativeInstance) };
    }
}

impl tTJSVariant {
    pub fn new() -> Self {
        type Type = extern "system" fn(s: *mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr3fc0c32ee41ea0c515f8fbb681e37982,
                "tTJSVariant::tTJSVariant()\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my);
        my
    }

    pub fn change_closure_obj_this(&mut self, objthis: *mut iTJSDispatch2) {
        type Type = extern "system" fn(*mut tTJSVariant, *mut iTJSDispatch2);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr3d4b725f0b4234d79524822e7c34486b,
                "void tTJSVariant::ChangeClosureObjThis(iTJSDispatch2 *)\0",
                Type
            )
        };
        ptr(self, objthis)
    }

    pub fn typ_mut(&mut self) -> tTJSVariantType {
        type Type = extern "system" fn(*mut tTJSVariant) -> tTJSVariantType;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr8fca7d3a123df1eacf228ba89f6a02ff,
                "tTJSVariantType tTJSVariant::Type()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn clear(&mut self) {
        type Type = extern "system" fn(s: *mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr58be195f96a36c158d638e3b0c79308b,
                "void tTJSVariant::Clear()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_object_closure(&mut self) -> &mut tTJSVariantClosure {
        type Type = extern "system" fn(*mut tTJSVariant) -> *mut tTJSVariantClosure;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtreaa4d5b1d186a807a63311ab6d5e16e4,
                "tTJSVariantClosure & tTJSVariant::AsObjectClosure()\0",
                Type
            )
        };
        unsafe { &mut *ptr(self) }
    }

    pub fn to_object(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr246f30d208c1d3a4e2b558090f403734,
                "void tTJSVariant::ToObject()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn to_string(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrc5a30d297c3a121879b1392bc6c604ef,
                "void tTJSVariant::ToString()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn get_hint(&mut self) -> *mut tjs_uint32 {
        type Type = extern "system" fn(*mut tTJSVariant) -> *mut tjs_uint32;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtre398f5aef0ab92bc1323f3b094722fb1,
                "tjs_uint32 * tTJSVariant::GetHint()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_object(&self) -> *mut iTJSDispatch2 {
        type Type = extern "system" fn(s: *const tTJSVariant) -> *mut iTJSDispatch2;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr841ce4492b37321eea0c1b500de9b352,
                "iTJSDispatch2 * tTJSVariant::AsObject() const\0",
                Type
            )
        };
        ptr(self)
    }
}

impl From<&tTJSVariant> for tTJSVariant {
    fn from(value: &tTJSVariant) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, ref_: *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtre8dbd4fe012262d9da831e0735aa33b3,
                "tTJSVariant::tTJSVariant(const tTJSVariant &)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value);
        my
    }
}

impl From<*mut iTJSDispatch2> for tTJSVariant {
    fn from(value: *mut iTJSDispatch2) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, ref_: *mut iTJSDispatch2);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrace6cce1353865d7376caca1f2124216,
                "tTJSVariant::tTJSVariant(iTJSDispatch2 *)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value);
        my
    }
}

impl From<(*mut iTJSDispatch2, *mut iTJSDispatch2)> for tTJSVariant {
    fn from(value: (*mut iTJSDispatch2, *mut iTJSDispatch2)) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, *mut iTJSDispatch2, *mut iTJSDispatch2);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr5055344aa8055bc238b79e5f88fc3300,
                "tTJSVariant::tTJSVariant(iTJSDispatch2 *,iTJSDispatch2 *)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value.0, value.1);
        my
    }
}

impl From<*const tjs_char> for tTJSVariant {
    fn from(value: *const tjs_char) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, *const tjs_char);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr8238c542b814acf1a83c00cced57ba26,
                "tTJSVariant::tTJSVariant(const tjs_char *)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value);
        my
    }
}

impl From<&tTJSString> for tTJSVariant {
    fn from(value: &tTJSString) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSString);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrbd2a14ca8c345fd7f151b08d1792fb60,
                "tTJSVariant::tTJSVariant(const tTJSString &)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value as *const _);
        my
    }
}

impl From<*const tjs_nchar> for tTJSVariant {
    fn from(value: *const tjs_nchar) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, *const tjs_nchar);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr16d432f9f86738a7688cbfc9b12441ec,
                "tTJSVariant::tTJSVariant(const tjs_nchar *)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value);
        my
    }
}

impl From<(*const tjs_uint8, tjs_uint)> for tTJSVariant {
    fn from(value: (*const tjs_uint8, tjs_uint)) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, *const tjs_uint8, tjs_uint);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr6dac00582b8ba529e548ef058c4e869e,
                "tTJSVariant::tTJSVariant(const tjs_uint8 *,tjs_uint)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value.0, value.1);
        my
    }
}

impl From<bool> for tTJSVariant {
    fn from(value: bool) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, bool);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr9193ae470b5efdfe617b5e94cd8f5da6,
                "tTJSVariant::tTJSVariant(bool)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value);
        my
    }
}

impl From<tjs_int32> for tTJSVariant {
    fn from(value: tjs_int32) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, tjs_int32);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrec455b6ef0f5da178063db3875973260,
                "tTJSVariant::tTJSVariant(tjs_int32)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value);
        my
    }
}

impl From<tjs_int64> for tTJSVariant {
    fn from(value: tjs_int64) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, tjs_int64);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtra56aaf685bd171b63b0ef3c894d80ecf,
                "tTJSVariant::tTJSVariant(tjs_int64)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value);
        my
    }
}

impl From<tjs_real> for tTJSVariant {
    fn from(value: tjs_real) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, tjs_real);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr9a5fe199cebb9841f94ac0bb7a4a3b6a,
                "tTJSVariant::tTJSVariant(tjs_real)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value);
        my
    }
}

impl From<*const *const tjs_uint8> for tTJSVariant {
    fn from(value: *const *const tjs_uint8) -> Self {
        type Type = extern "system" fn(*mut tTJSVariant, *const *const tjs_uint8);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr2acb76a1f86e34afc5fe934d406c6c4c,
                "tTJSVariant::tTJSVariant(const tjs_uint8 * *)\0",
                Type
            )
        };
        let mut my = tTJSVariant {
            _base: tTJSVariant_S::default(),
        };
        ptr(&mut my, value);
        my
    }
}

impl Default for tTJSVariant {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for tTJSVariant {
    fn drop(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr3a4d914ca7d24989c236ad223c002d49,
                "tTJSVariant::~ tTJSVariant()\0",
                Type
            )
        };
        ptr(self)
    }
}

impl Into<*mut iTJSDispatch2> for &mut tTJSVariant {
    fn into(self) -> *mut iTJSDispatch2 {
        type Type = extern "system" fn(*mut tTJSVariant) -> *mut iTJSDispatch2;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr3206ef9b7a8013d6572decdea49e7e2e,
                "tTJSVariant::operator iTJSDispatch2 *()\0",
                Type
            )
        };
        ptr(self)
    }
}
