//! Warpped types for tpstub
use crate::tp_stub::*;
use std::ops::Deref;

pub struct Octet {
    value: *mut tTJSVariantOctet,
}

impl Octet {
    /// p must already called add_ref
    pub unsafe fn new_owned(p: *mut tTJSVariantOctet) -> Self {
        Self { value: p }
    }

    pub unsafe fn raw(&self) -> *mut tTJSVariantOctet {
        self.value
    }
}

impl std::fmt::Debug for Octet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl std::fmt::Display for Octet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<% ")?;
        for b in self.iter() {
            write!(f, "{:02x} ", b)?;
        }
        f.write_str("%>")
    }
}

impl Deref for Octet {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        let r = unsafe { self.value.as_ref() };
        if let Some(r) = r {
            let data = r.get_data();
            let len = r.get_length() as usize;
            unsafe { std::slice::from_raw_parts(data, len) }
        } else {
            &[]
        }
    }
}

impl Drop for Octet {
    fn drop(&mut self) {
        let p = unsafe { self.value.as_mut() };
        if let Some(p) = p {
            p.release();
        }
    }
}
