use anyhow::Result;
use krkrz_plugin_base::{tp_stub::*, types::*, *};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufReader, Read};
use std::ptr;
use std::sync::{Mutex, MutexGuard};
use zip::ZipArchive;
use zip::result::ZipError;

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
static mut FS: *mut iTVPStorageMedia = ptr::null_mut();

type Archive = ZipArchive<BufReader<File>>;

struct ZipFs {
    archives: Mutex<HashMap<String, Archive>>,
}

impl ZipFs {
    fn new() -> Self {
        Self {
            archives: Mutex::new(HashMap::new()),
        }
    }

    fn open_archive(name: &str) -> Result<Archive> {
        let file = BufReader::new(File::open(name)?);
        Ok(ZipArchive::new(file)?)
    }

    fn check_exist<P: AsRef<std::path::Path>>(name: &P) -> bool {
        match std::fs::exists(name) {
            Ok(s) => s,
            Err(e) => {
                log!(
                    "Failed to check file {} exists: {e}",
                    name.as_ref().display()
                );
                false
            }
        }
    }

    fn get_archive<'a, 'b>(
        archives: &'a mut MutexGuard<'b, HashMap<String, Archive>>,
        name: &str,
    ) -> Result<Option<&'a mut Archive>> {
        if !archives.contains_key(name) {
            if std::fs::exists(name)? {
                let archive = Self::open_archive(name)?;
                archives.insert(name.to_string(), archive);
            } else {
                return Ok(None);
            }
        }
        Ok(archives.get_mut(name))
    }

    fn index_for_name_ignore_case(archive: &Archive, name: &str) -> Option<usize> {
        let name = name.replace('\\', "/");
        archive
            .file_names()
            .position(|entry| entry.eq_ignore_ascii_case(&name))
    }

    fn split_storage_name(name: &str) -> Option<(&str, &str)> {
        let (zipname, path) = name.split_once('/')?;
        Some((zipname, path.trim_start_matches('/')))
    }

    fn list_at(archive: &Archive, path: &str) -> Vec<String> {
        let path = path.replace('\\', "/");
        let path_parts: Vec<_> = path.split('/').filter(|part| !part.is_empty()).collect();
        let mut entries = BTreeMap::new();

        for entry_name in archive.file_names() {
            let entry_name = entry_name.replace('\\', "/");
            let entry_parts: Vec<_> = entry_name
                .split('/')
                .filter(|part| !part.is_empty())
                .collect();
            if entry_parts.len() <= path_parts.len()
                || !path_parts
                    .iter()
                    .zip(&entry_parts)
                    .all(|(path_part, entry_part)| path_part.eq_ignore_ascii_case(entry_part))
            {
                continue;
            }

            let child = entry_parts[path_parts.len()];
            let is_dir = entry_parts.len() > path_parts.len() + 1 || entry_name.ends_with('/');
            let display_name = if is_dir {
                format!("{child}/")
            } else {
                child.to_string()
            };
            entries
                .entry(child.to_ascii_lowercase())
                .or_insert(display_name);
        }

        entries.into_values().collect()
    }
}

impl TVPStorageMedia for ZipFs {
    fn get_name(&mut self, name: &mut ttstr) {
        name.assign("zip");
    }

    fn normalize_domain_name(&mut self, name: &mut ttstr) {
        log!("normalize_domain_name: {}", name.to_string());
        name.to_lower_case();
    }

    fn normalize_path_name(&mut self, name: &mut ttstr) {
        log!("normalize_path_name: {}", name.to_string());
    }

    fn check_existent_storage(&mut self, name: &ttstr) -> bool {
        let name = name.to_string();
        log!("check_existent_storage: {}", name);
        // name is domain/path
        let Some((zipname, path)) = Self::split_storage_name(&name) else {
            return false;
        };
        let mut archives = self.archives.lock().unwrap();
        if let Some(archive) = archives.get(zipname) {
            return Self::index_for_name_ignore_case(archive, path).is_some();
        }
        if Self::check_exist(&zipname) {
            match Self::open_archive(zipname) {
                Ok(zip) => {
                    let r = Self::index_for_name_ignore_case(&zip, path).is_some();
                    archives.insert(zipname.to_owned(), zip);
                    return r;
                }
                Err(e) => {
                    log!("Failed to open zip file {}: {}", zipname, e);
                }
            }
        }
        false
    }

    fn open(&mut self, name: &ttstr, flags: tjs_uint32) -> *mut tTJSBinaryStream {
        if flags != TJS_BS_READ {
            return ptr::null_mut();
        }
        let name = name.to_string();
        log!("open: {}", name);
        // name is domain/path
        let Some((zipname, path)) = Self::split_storage_name(&name) else {
            return ptr::null_mut();
        };
        let mut archives = self.archives.lock().unwrap();
        let zip = match Self::get_archive(&mut archives, zipname) {
            Ok(None) => return ptr::null_mut(),
            Ok(Some(zip)) => zip,
            Err(e) => {
                log!("Failed to open zip file {zipname}: {e}");
                return ptr::null_mut();
            }
        };
        let index = match Self::index_for_name_ignore_case(zip, path) {
            Some(i) => i,
            None => {
                return ptr::null_mut();
            }
        };
        match zip.by_index_seek(index) {
            Ok(z) => {
                let s = com::create_istream_read_seek(z);
                let r = unsafe { TVPCreateBinaryStreamAdapter(s) };
                unsafe { (*s).release() };
                return r;
            }
            Err(ZipError::UnsupportedArchive(_)) => {}
            Err(e) => {
                log!("Failed to open file {path} in zip {zipname}: {e}");
                return ptr::null_mut();
            }
        }
        match zip.by_index(index) {
            Ok(mut z) => {
                let mut b = Vec::new();
                if let Err(e) = z.read_to_end(&mut b) {
                    log!("Failed to read file {path} in zip {zipname}: {e}");
                    return ptr::null_mut();
                }
                let s = com::create_istream_read_seek(std::io::Cursor::new(b));
                let r = unsafe { TVPCreateBinaryStreamAdapter(s) };
                unsafe { (*s).release() };
                r
            }
            Err(e) => {
                log!("Failed to open file {path} in zip {zipname}: {e}");
                ptr::null_mut()
            }
        }
    }

    fn get_list_at(&mut self, name: &ttstr, lister: *mut iTVPStorageLister) {
        let name = name.to_string();
        log!("get_list_at: {}", name);
        if lister.is_null() || unsafe { (*lister).vtable_ }.is_null() {
            return;
        }
        let Some((zipname, path)) = Self::split_storage_name(&name) else {
            return;
        };
        let mut archives = self.archives.lock().unwrap();
        let zip = match Self::get_archive(&mut archives, zipname) {
            Ok(None) => return,
            Ok(Some(zip)) => zip,
            Err(e) => {
                log!("Failed to open zip file {zipname}: {e}");
                return;
            }
        };
        let entries = Self::list_at(zip, path);
        for entry in entries {
            let file = ttstr::from(entry.as_str());
            unsafe {
                (*lister).add(&file);
            }
        }
    }

    fn get_locally_accessible_name(name: &mut ttstr) {
        name.assign("");
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let fs = create_tvp_storage_media(ZipFs::new());
    unsafe { FS = fs };
    unsafe { TVPRegisterStorageMedia(fs) };
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[zipfs]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unsafe { TVPUnregisterStorageMedia(FS) };
    unsafe { (*FS).release() }
    unsafe { FS = ptr::null_mut() };
    unsafe { TVPUninitImportStub() };
    0
}
