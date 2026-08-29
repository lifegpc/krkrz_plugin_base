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

pub trait TVPStorageMedia {
    /// returns media name like "file", "http" etc.
    fn get_name(&mut self, name: &mut ttstr);
    /// normalize domain name according with the media's rule
    fn normalize_domain_name(&mut self, name: &mut ttstr);
    /// normalize path name according with the media's rule
    /// "name" below is normalized but does not contain media, eg.
    /// not "media://domain/path" but "domain/path"
    fn normalize_path_name(&mut self, name: &mut ttstr);
    /// check file existence
    fn check_existent_storage(&mut self, name: &ttstr) -> bool;
    /// open a storage and return a tTJSBinaryStream instance.
    /// name does not contain in-archive storage name but
    /// is normalized.
    fn open(&mut self, name: &ttstr, flags: tjs_uint32) -> *mut tTJSBinaryStream;
    /// list files at given place
    fn get_list_at(&mut self, name: &ttstr, lister: *mut iTVPStorageLister);
    /// basically the same as above,
    /// check wether given name is easily accessible from local OS filesystem.
    /// if true, returns local OS native name. otherwise returns an empty string.
    fn get_locally_accessible_name(name: &mut ttstr);
}

#[repr(C)]
struct TVPStorageMediaWrapper<T> {
    base: iTVPStorageMedia,
    ref_: tjs_uint,
    media: T,
}

impl<T: TVPStorageMedia> TVPStorageMediaWrapper<T> {
    const VTABLE: iTVPStorageMedia__bindgen_vtable = iTVPStorageMedia__bindgen_vtable {
        iTVPStorageMedia_AddRef: Self::add_ref,
        iTVPStorageMedia_Release: Self::release,
        iTVPStorageMedia_GetName: Self::get_name,
        iTVPStorageMedia_NormalizeDomainName: Self::normalize_domain_name,
        iTVPStorageMedia_NormalizePathName: Self::normalize_path_name,
        iTVPStorageMedia_CheckExistentStorage: Self::check_existent_storage,
        iTVPStorageMedia_Open: Self::open,
        iTVPStorageMedia_GetListAt: Self::get_list_at,
        iTVPStorageMedia_GetLocallyAccessibleName: Self::get_locally_accessible_name,
    };

    fn new(media: T) -> *mut iTVPStorageMedia {
        let boxed = Box::new(Self {
            base: iTVPStorageMedia {
                vtable_: &Self::VTABLE,
            },
            ref_: 1,
            media,
        });
        unsafe {
            TVPPluginGlobalRefCount += 1;
        }
        Box::into_raw(boxed) as *mut iTVPStorageMedia
    }

    unsafe fn from_base<'a>(this: *mut iTVPStorageMedia) -> &'a mut Self {
        unsafe { &mut *(this as *mut Self) }
    }

    unsafe extern "C" fn add_ref(this: *mut iTVPStorageMedia) {
        let self_ = unsafe { Self::from_base(this) };
        self_.ref_ += 1;
        unsafe {
            TVPPluginGlobalRefCount += 1;
        }
    }

    unsafe extern "C" fn release(this: *mut iTVPStorageMedia) {
        unsafe {
            TVPPluginGlobalRefCount -= 1;
        }
        let self_ = unsafe { Self::from_base(this) };
        if self_.ref_ == 1 {
            unsafe { drop(Box::from_raw(this as *mut Self)) };
        } else {
            self_.ref_ -= 1;
        }
    }

    unsafe extern "C" fn get_name(this: *mut iTVPStorageMedia, name: *mut ttstr) {
        unsafe { Self::from_base(this) }
            .media
            .get_name(unsafe { &mut *name });
    }

    unsafe extern "C" fn normalize_domain_name(this: *mut iTVPStorageMedia, name: *mut ttstr) {
        unsafe { Self::from_base(this) }
            .media
            .normalize_domain_name(unsafe { &mut *name });
    }

    unsafe extern "C" fn normalize_path_name(this: *mut iTVPStorageMedia, name: *mut ttstr) {
        unsafe { Self::from_base(this) }
            .media
            .normalize_path_name(unsafe { &mut *name });
    }

    unsafe extern "C" fn check_existent_storage(
        this: *mut iTVPStorageMedia,
        name: *const ttstr,
    ) -> bool {
        unsafe { Self::from_base(this) }
            .media
            .check_existent_storage(unsafe { &*name })
    }

    unsafe extern "C" fn open(
        this: *mut iTVPStorageMedia,
        name: *const ttstr,
        flags: tjs_uint32,
    ) -> *mut tTJSBinaryStream {
        unsafe { Self::from_base(this) }
            .media
            .open(unsafe { &*name }, flags)
    }

    unsafe extern "C" fn get_list_at(
        this: *mut iTVPStorageMedia,
        name: *const ttstr,
        lister: *mut iTVPStorageLister,
    ) {
        unsafe { Self::from_base(this) }
            .media
            .get_list_at(unsafe { &*name }, lister);
    }

    unsafe extern "C" fn get_locally_accessible_name(
        this: *mut iTVPStorageMedia,
        name: *mut ttstr,
    ) {
        let _ = this;
        T::get_locally_accessible_name(unsafe { &mut *name });
    }
}

pub fn create_tvp_storage_media<T: TVPStorageMedia>(v: T) -> *mut iTVPStorageMedia {
    TVPStorageMediaWrapper::new(v)
}
