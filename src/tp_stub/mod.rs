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

    pub fn to_octet(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr0733b0ac80880897d327dc6f3b04ea9e,
                "void tTJSVariant::ToOctet()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn to_integer(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr4cb055ed9d8ef71d1af10898965c940c,
                "void tTJSVariant::ToInteger()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn to_real(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtref8d198596b7d3143d02ed4450ccefa1,
                "void tTJSVariant::ToReal()\0",
                Type
            )
        };
        ptr(self)
    }

    #[inline(always)]
    pub fn assign<T>(&mut self, rhs: T) -> &mut Self
    where
        Self: Assign<T>,
    {
        Assign::assign(self, rhs)
    }

    pub fn copy_ref(&mut self, ref_: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr679b215ff76a269871d5f325b981e561,
                "void tTJSVariant::CopyRef(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, ref_)
    }

    pub fn set_object(&mut self, ref_: *mut iTJSDispatch2) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, *mut iTJSDispatch2) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr2f873b0ee1c6591ba28bc4b9c0e4c954,
                "tTJSVariant & tTJSVariant::SetObject(iTJSDispatch2 *)\0",
                Type
            )
        };
        let p = ptr(self, ref_);
        unsafe { &mut *p }
    }

    pub fn set_object2(
        &mut self,
        object: *mut iTJSDispatch2,
        objthis: *mut iTJSDispatch2,
    ) -> &mut Self {
        type Type = extern "system" fn(
            *mut tTJSVariant,
            *mut iTJSDispatch2,
            *mut iTJSDispatch2,
        ) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtra583ffb56cdb2ede691e15053a8a165a,
                "tTJSVariant & tTJSVariant::SetObject(iTJSDispatch2 *,iTJSDispatch2 *)\0",
                Type
            )
        };
        let p = ptr(self, object, objthis);
        unsafe { &mut *p }
    }

    pub fn logical_or_equal_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr6efc1d1f66f0e01a81faf767d7576816,
                "void tTJSVariant::logicalorequal(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn logical_and_equal_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr4ededf58eae77c320b4a6f5f701acafb,
                "void tTJSVariant::logicalandequal(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn increment(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr11912984b8c094d2df26bf3c3677d096,
                "void tTJSVariant::increment()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn decrement(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr6c0df790c33142e286aea9af6993d931,
                "void tTJSVariant::decrement()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn rbit_shift_equal_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr786a65424247e711f6ca31f0a10603d7,
                "void tTJSVariant::rbitshiftequal(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn idiv_equal_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr17cbcacad2ed350215d7d700c676ea40,
                "void tTJSVariant::idivequal(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn logicalnot(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr4d2c157f8b0b49e57c3e9b5abc9deb0f,
                "void tTJSVariant::logicalnot()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn bitnot(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr4b7eaccf64af0f3a4c4fe64f4e2dd3fd,
                "void tTJSVariant::bitnot()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn tonumber(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr3a4d2602c392a8d1f4c38d537a8c95e0,
                "void tTJSVariant::tonumber()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn changesign(&mut self) {
        type Type = extern "system" fn(*mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr8d915d35ef8e857f245c5d46798618e4,
                "void tTJSVariant::changesign()\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn typ(&self) -> tTJSVariantType {
        type Type = extern "system" fn(*const tTJSVariant) -> tTJSVariantType;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr972e0f9a6ec4648a9fb82bcf5d9095ff,
                "tTJSVariantType tTJSVariant::Type() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn normal_compare(&self, val2: &tTJSVariant) -> bool {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr9d76731c37c4664d654db026644c64b4,
                "bool tTJSVariant::NormalCompare(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, val2)
    }

    pub fn discern_compare(&self, val2: &tTJSVariant) -> bool {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr4f1620cb699874b9c8cedf6e321c606e,
                "bool tTJSVariant::DiscernCompare(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, val2)
    }

    pub fn discern_compare_strict_real(&self, val2: &tTJSVariant) -> bool {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtref1c6b2b601d1b0ff70272a4d447aa3c,
                "bool tTJSVariant::DiscernCompareStrictReal(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, val2)
    }

    pub fn greater_than(&self, val2: &tTJSVariant) -> bool {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr9b7872860c95cfdafb056ab30318e99c,
                "bool tTJSVariant::GreaterThan(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, val2)
    }

    pub fn littler_than(&self, val2: &tTJSVariant) -> bool {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr53360f194a04fc142ddae2b9a3ab4c92,
                "bool tTJSVariant::LittlerThan(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, val2)
    }

    pub fn is_instance_of(&self, classname: *const tjs_char) -> bool {
        type Type = extern "system" fn(*const tTJSVariant, *const tjs_char) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrce1dcb05e5e7c4cafbc4ed37f63b256e,
                "bool tTJSVariant::IsInstanceOf(const tjs_char *) const\0",
                Type
            )
        };
        ptr(self, classname)
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

    pub fn as_object_no_add_ref(&self) -> *mut iTJSDispatch2 {
        type Type = extern "system" fn(*const tTJSVariant) -> *mut iTJSDispatch2;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr61785de870894968cd9d95e17e88eafc,
                "iTJSDispatch2 * tTJSVariant::AsObjectNoAddRef() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_object_this(&self) -> *mut iTJSDispatch2 {
        type Type = extern "system" fn(*const tTJSVariant) -> *mut iTJSDispatch2;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrad3236e727398311c3b8e1ddd5f4b293,
                "iTJSDispatch2 * tTJSVariant::AsObjectThis() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_object_this_no_add_ref(&self) -> *mut iTJSDispatch2 {
        type Type = extern "system" fn(*const tTJSVariant) -> *mut iTJSDispatch2;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr80e0b7be488545ff9b8bc52c9ab5fba5,
                "iTJSDispatch2 * tTJSVariant::AsObjectThisNoAddRef() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_object_closure_no_add_ref(&self) -> &tTJSVariantClosure {
        type Type = extern "system" fn(*const tTJSVariant) -> *const tTJSVariantClosure;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr4eaa3e4efb319707db6ef81db1c6f147,
                "tTJSVariantClosure & tTJSVariant::AsObjectClosureNoAddRef() const\0",
                Type
            )
        };
        unsafe { &*ptr(self) }
    }

    pub fn as_string(&self) -> *mut tTJSVariantString {
        type Type = extern "system" fn(*const tTJSVariant) -> *mut tTJSVariantString;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr693a0152f098caee7fc77f545dd3e954,
                "tTJSVariantString * tTJSVariant::AsString() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_string_no_add_ref(&self) -> *mut tTJSVariantString {
        type Type = extern "system" fn(*const tTJSVariant) -> *mut tTJSVariantString;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr42840710f5fba9bb32b95290b1796a55,
                "tTJSVariantString * tTJSVariant::AsStringNoAddRef() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn get_string(&self) -> *const tjs_char {
        type Type = extern "system" fn(*const tTJSVariant) -> *const tjs_char;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtradec3f9ef429aa9a284081f0fc6a1b5b,
                "const tjs_char * tTJSVariant::GetString() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_octet(&self) -> *mut tTJSVariantOctet {
        type Type = extern "system" fn(*const tTJSVariant) -> *mut tTJSVariantOctet;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr674a7948152a1d7a49050b9d98796403,
                "tTJSVariantOctet * tTJSVariant::AsOctet() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_octet_no_add_ref(&self) -> *mut tTJSVariantOctet {
        type Type = extern "system" fn(*const tTJSVariant) -> *mut tTJSVariantOctet;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtraa6f132b2031c83062f6149c90f2df5f,
                "tTJSVariantOctet * tTJSVariant::AsOctetNoAddRef() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_integer(&self) -> tTVInteger {
        type Type = extern "system" fn(*const tTJSVariant) -> tTVInteger;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrb52f446e22bb92d495f7e65ac71c9bf9,
                "tTVInteger tTJSVariant::AsInteger() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn as_number(&self, targ: &mut tTJSVariant) {
        type Type = extern "system" fn(*const tTJSVariant, *mut tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrd4899fd4a8beb06f192dcb1d300e3319,
                "void tTJSVariant::AsNumber(tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, targ)
    }

    pub fn as_real(&self) -> tTVReal {
        type Type = extern "system" fn(*const tTJSVariant) -> tTVReal;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtra5f80951cfb882ac6a3e06c0b9a95807,
                "tTVReal tTJSVariant::AsReal() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn logical_or(&self, rhs: &tTJSVariant) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrfb6573df5887c2020ae58136f8342ed4,
                "tTJSVariant tTJSVariant::operator ||(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn logical_and(&self, rhs: &tTJSVariant) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr86c67d2197c46824ab10f59e568ad13a,
                "tTJSVariant tTJSVariant::operator &&(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn equal(&self, rhs: &tTJSVariant) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr5d91cff3b2a26ff7c0543e0f6d737117,
                "tTJSVariant tTJSVariant::operator ==(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn not_equal(&self, rhs: &tTJSVariant) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr9996100acc7705cb2b0c904d6bad4401,
                "tTJSVariant tTJSVariant::operator !=(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn less(&self, rhs: &tTJSVariant) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtref1dedc2cb58dc4e1afc14238b6fc518,
                "tTJSVariant tTJSVariant::operator <(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn greater(&self, rhs: &tTJSVariant) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrf18397fe81c043ba2346e31b359f6a73,
                "tTJSVariant tTJSVariant::operator >(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn less_equal(&self, rhs: &tTJSVariant) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr2ee45ad60b0c06a8d0feebc3a6aad9e7,
                "tTJSVariant tTJSVariant::operator <=(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn greater_equal(&self, rhs: &tTJSVariant) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr44500491c57e17032951fe6ed268ff1d,
                "tTJSVariant tTJSVariant::operator >=(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn rbit_shift(&self, count: tjs_int) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant, tjs_int) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr04233bc4f7d4df92c260d23110320afe,
                "tTJSVariant tTJSVariant::rbitshift(tjs_int) const\0",
                Type
            )
        };
        ptr(self, count)
    }

    pub fn idiv(&self, rhs: &tTJSVariant) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr02164e6fb4c925843ac774ec1e4c6e5d,
                "tTJSVariant tTJSVariant::idiv(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }

    pub fn unary_plus(&self) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrba7ff7b0b4192bd2cc7f49c7b688ad57,
                "tTJSVariant tTJSVariant::operator +() const\0",
                Type
            )
        };
        ptr(self)
    }

    pub fn bit_not(&self) -> tTJSVariant {
        type Type = extern "system" fn(*const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr9d0edd8f51f155767301017bd3d256da,
                "tTJSVariant tTJSVariant::operator ~() const\0",
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

impl Into<tTVInteger> for &tTJSVariant {
    fn into(self) -> tTVInteger {
        type Type = extern "system" fn(*const tTJSVariant) -> tTVInteger;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrd3f5ec78464d29ee6988a1f90c2e3e1b,
                "tTJSVariant::operator tTVInteger() const\0",
                Type
            )
        };
        ptr(self)
    }
}

impl Into<bool> for &tTJSVariant {
    fn into(self) -> bool {
        type Type = extern "system" fn(*const tTJSVariant) -> bool;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtra463ad6a757c3f04e09a72e288737d06,
                "tTJSVariant::operator bool() const\0",
                Type
            )
        };
        ptr(self)
    }
}

impl Into<tjs_int> for &tTJSVariant {
    fn into(self) -> tjs_int {
        type Type = extern "system" fn(*const tTJSVariant) -> tjs_int;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr27857bb89d35113183b682c3917d6c7a,
                "tTJSVariant::operator tjs_int() const\0",
                Type
            )
        };
        ptr(self)
    }
}

impl Into<tTVReal> for &tTJSVariant {
    fn into(self) -> tTVReal {
        type Type = extern "system" fn(*const tTJSVariant) -> tTVReal;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr35aadb63079c8bd84ebc0389bae306e0,
                "tTJSVariant::operator tTVReal() const\0",
                Type
            )
        };
        ptr(self)
    }
}

impl Assign<&tTJSVariant> for tTJSVariant {
    fn assign(&mut self, rhs: &tTJSVariant) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrd48ea419e040ffe8c20c1e86d80c9a5f,
                "tTJSVariant & tTJSVariant::operator =(const tTJSVariant &)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<*mut iTJSDispatch2> for tTJSVariant {
    fn assign(&mut self, rhs: *mut iTJSDispatch2) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, *mut iTJSDispatch2) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr1039eff4a4443f9238438485a35a93a7,
                "tTJSVariant & tTJSVariant::operator =(iTJSDispatch2 *)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<tTJSVariantClosure> for tTJSVariant {
    fn assign(&mut self, rhs: tTJSVariantClosure) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, tTJSVariantClosure) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtre09ed277802c1b117e1908421448886d,
                "tTJSVariant & tTJSVariant::operator =(tTJSVariantClosure)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<*mut tTJSVariantString> for tTJSVariant {
    fn assign(&mut self, rhs: *mut tTJSVariantString) -> &mut Self {
        type Type =
            extern "system" fn(*mut tTJSVariant, *mut tTJSVariantString) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtre76dfb9e00f4a9d491117d815f30db7f,
                "tTJSVariant & tTJSVariant::operator =(tTJSVariantString *)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<*mut tTJSVariantOctet> for tTJSVariant {
    fn assign(&mut self, rhs: *mut tTJSVariantOctet) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, *mut tTJSVariantOctet) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrb000dd8934508d8ec6d6ef976a6ff49b,
                "tTJSVariant & tTJSVariant::operator =(tTJSVariantOctet *)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<&tTJSString> for tTJSVariant {
    fn assign(&mut self, rhs: &tTJSString) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSString) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrd98ab5c968ebfde4e924901d09190774,
                "tTJSVariant & tTJSVariant::operator =(const tTJSString &)\0",
                Type
            )
        };
        let re = ptr(self, rhs as *const _);
        unsafe { &mut *re }
    }
}

impl Assign<*const tjs_char> for tTJSVariant {
    fn assign(&mut self, rhs: *const tjs_char) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, *const tjs_char) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr661e8c10d5d477e6823a840244937cd8,
                "tTJSVariant & tTJSVariant::operator =(const tjs_char *)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<&[tjs_char]> for tTJSVariant {
    fn assign(&mut self, rhs: &[tjs_char]) -> &mut Self {
        self.assign(rhs.as_ptr())
    }
}

impl Assign<*const tjs_nchar> for tTJSVariant {
    fn assign(&mut self, rhs: *const tjs_nchar) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, *const tjs_nchar) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr6b39e70ea89c4f883689f51289029b69,
                "tTJSVariant & tTJSVariant::operator =(const tjs_nchar *)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<&[tjs_nchar]> for tTJSVariant {
    fn assign(&mut self, rhs: &[tjs_nchar]) -> &mut Self {
        self.assign(rhs.as_ptr())
    }
}

impl Assign<&[u8]> for tTJSVariant {
    fn assign(&mut self, rhs: &[u8]) -> &mut Self {
        self.assign(rhs.as_ptr() as *const i8)
    }
}

impl Assign<&str> for tTJSVariant {
    fn assign(&mut self, rhs: &str) -> &mut Self {
        let mut encoded: Vec<_> = rhs.encode_utf16().collect();
        encoded.push(0);
        self.assign(encoded.as_ptr())
    }
}

impl Assign<bool> for tTJSVariant {
    fn assign(&mut self, rhs: bool) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, bool) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr4a18b1c0afe37b84e2b35a7fc07c4e0f,
                "tTJSVariant & tTJSVariant::operator =(bool)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<tjs_int32> for tTJSVariant {
    fn assign(&mut self, rhs: tjs_int32) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, tjs_int32) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr48b85c8774d91ca40b2992f0e452f19e,
                "tTJSVariant & tTJSVariant::operator =(tjs_int32)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<tTVInteger> for tTJSVariant {
    fn assign(&mut self, rhs: tTVInteger) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, tTVInteger) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr5ea8db9a9193fe6bab53baf2bee06b6b,
                "tTJSVariant & tTJSVariant::operator =(const tTVInteger)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl Assign<tjs_real> for tTJSVariant {
    fn assign(&mut self, rhs: tjs_real) -> &mut Self {
        type Type = extern "system" fn(*mut tTJSVariant, tjs_real) -> *mut tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr46b92626ff6894e993c4f193a129540b,
                "tTJSVariant & tTJSVariant::operator =(tjs_real)\0",
                Type
            )
        };
        let re = ptr(self, rhs);
        unsafe { &mut *re }
    }
}

impl BitOrAssign<&tTJSVariant> for tTJSVariant {
    fn bitor_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr028d5fda2f4568f6ab14b49d89650a4d,
                "void tTJSVariant::operator |=(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl BitXorAssign<&tTJSVariant> for tTJSVariant {
    fn bitxor_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrc27d85b695cd6e144210785bdfd446ce,
                "void tTJSVariant::operator ^=(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl BitAndAssign<&tTJSVariant> for tTJSVariant {
    fn bitand_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr8422ef7f42009be0ad58a09d64149051,
                "void tTJSVariant::operator &=(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl ShrAssign<&tTJSVariant> for tTJSVariant {
    fn shr_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtree07e6522577952453206ede39cdf54c,
                "void tTJSVariant::operator >>=(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl ShlAssign<&tTJSVariant> for tTJSVariant {
    fn shl_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr995a222f2038dd2007f2c1f6429bd19e,
                "void tTJSVariant::operator <<=(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl RemAssign<&tTJSVariant> for tTJSVariant {
    fn rem_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrda8c6e750d6a9c0557a56ef7f7fd8e88,
                "void tTJSVariant::operator %=(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl DivAssign<&tTJSVariant> for tTJSVariant {
    fn div_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr9cf7b0f119bcf3fa4564837ae25429b3,
                "void tTJSVariant::operator /=(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl MulAssign<&tTJSVariant> for tTJSVariant {
    fn mul_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr2bd375c0598e9148d88579a51b2f07a8,
                "void tTJSVariant::operator *=(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl SubAssign<&tTJSVariant> for tTJSVariant {
    fn sub_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr1e463482afa8ca30f5fa7bea4fa5741d,
                "void tTJSVariant::operator -=(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl AddAssign<&tTJSVariant> for tTJSVariant {
    fn add_assign(&mut self, rhs: &tTJSVariant) {
        type Type = extern "system" fn(*mut tTJSVariant, *const tTJSVariant);
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrfdf270e4080c986abd1649fa9fffdeab,
                "void tTJSVariant::operator +=(const tTJSVariant &)\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl BitOr<&tTJSVariant> for &tTJSVariant {
    type Output = tTJSVariant;
    fn bitor(self, rhs: &tTJSVariant) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr263a0c5b335b2c4d5bc1f55b51b8315e,
                "tTJSVariant tTJSVariant::operator |(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl BitXor<&tTJSVariant> for &tTJSVariant {
    type Output = tTJSVariant;
    fn bitxor(self, rhs: &tTJSVariant) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr975c1099e57ab67122ddef0f44fd7dd5,
                "tTJSVariant tTJSVariant::operator ^(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl BitAnd<&tTJSVariant> for &tTJSVariant {
    type Output = tTJSVariant;
    fn bitand(self, rhs: &tTJSVariant) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr04493e5237a7ca97afd391cb7e831ba0,
                "tTJSVariant tTJSVariant::operator &(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Shr<&tTJSVariant> for &tTJSVariant {
    type Output = tTJSVariant;
    fn shr(self, rhs: &tTJSVariant) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr056f5d278c75750df792bf8b081fbf7d,
                "tTJSVariant tTJSVariant::operator >>(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Shl<&tTJSVariant> for &tTJSVariant {
    type Output = tTJSVariant;
    fn shl(self, rhs: &tTJSVariant) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtrcdc475c4419e77c22508e337428c4074,
                "tTJSVariant tTJSVariant::operator <<(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Rem<&tTJSVariant> for &tTJSVariant {
    type Output = tTJSVariant;
    fn rem(self, rhs: &tTJSVariant) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr06bacb2910308a47bbe27ff7efa1226d,
                "tTJSVariant tTJSVariant::operator %(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Div<&tTJSVariant> for &tTJSVariant {
    type Output = tTJSVariant;
    fn div(self, rhs: &tTJSVariant) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr521e053199a4aeb4e0f24d9f4a6cc682,
                "tTJSVariant tTJSVariant::operator /(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Mul<&tTJSVariant> for &tTJSVariant {
    type Output = tTJSVariant;
    fn mul(self, rhs: &tTJSVariant) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr5110cbbcddbd9688281ee5418e3f9023,
                "tTJSVariant tTJSVariant::operator *(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Sub<&tTJSVariant> for &tTJSVariant {
    type Output = tTJSVariant;
    fn sub(self, rhs: &tTJSVariant) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr8f744c5aa8df5471939b960bc759f12b,
                "tTJSVariant tTJSVariant::operator -(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Add<&tTJSVariant> for &tTJSVariant {
    type Output = tTJSVariant;
    fn add(self, rhs: &tTJSVariant) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant, *const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr114a781ed71edace31abb352a2671f41,
                "tTJSVariant tTJSVariant::operator +(const tTJSVariant &) const\0",
                Type
            )
        };
        ptr(self, rhs)
    }
}

impl Not for &tTJSVariant {
    type Output = tTJSVariant;
    /// This is map to logical not. If you want to bit not. Call [`bit_not`](tTJSVariant::bit_not) instead.
    fn not(self) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr1db54b61f00bf931452218c4a39e79ef,
                "tTJSVariant tTJSVariant::operator !() const\0",
                Type
            )
        };
        ptr(self)
    }
}

impl Neg for &tTJSVariant {
    type Output = tTJSVariant;
    fn neg(self) -> Self::Output {
        type Type = extern "system" fn(*const tTJSVariant) -> tTJSVariant;
        let ptr = unsafe {
            import_func!(
                TVPImportFuncPtr7773ac921bb82c85de3be69ef86265fd,
                "tTJSVariant tTJSVariant::operator -() const\0",
                Type
            )
        };
        ptr(self)
    }
}
