use anyhow::{Context, Result, anyhow, bail};
use krkrz_plugin_base::{tp_stub::*, types::*, *};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::{Arc, Mutex};

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
static mut FS: *mut iTVPStorageMedia = ptr::null_mut();

#[derive(Clone, Debug)]
struct FileEntry {
    path: String,
    offset: u64,
    size: u64,
    unpacked: bool,
}

struct Archive {
    path: PathBuf,
    data_offset: u64,
    files: HashMap<String, FileEntry>,
    directories: HashMap<String, BTreeMap<String, String>>,
}

impl Archive {
    fn open(path: &Path) -> Result<Self> {
        let path = path
            .canonicalize()
            .with_context(|| format!("failed to resolve ASAR path {}", path.display()))?;
        let mut file =
            File::open(&path).with_context(|| format!("failed to open ASAR {}", path.display()))?;
        let file_len = file.metadata()?.len();
        if file_len < 8 {
            bail!("ASAR header is shorter than 8 bytes");
        }

        // ASAR starts with an 8-byte pickle prefix. The second u32 is the
        // length of the following header pickle, whose first u32 is the JSON
        // string length. The packed file data begins after that pickle.
        let mut prefix = [0u8; 8];
        file.read_exact(&mut prefix)?;
        let header_pickle_size = u32::from_le_bytes(prefix[4..8].try_into()?) as u64;
        let data_offset = 8u64
            .checked_add(header_pickle_size)
            .ok_or_else(|| anyhow!("ASAR header size overflow"))?;
        if data_offset > file_len || header_pickle_size < 8 {
            bail!("invalid ASAR header size {header_pickle_size}");
        }

        let pickle_len = usize::try_from(header_pickle_size)
            .context("ASAR header is too large for this platform")?;
        let mut header_pickle = Vec::new();
        header_pickle
            .try_reserve_exact(pickle_len)
            .context("failed to allocate ASAR header")?;
        header_pickle.resize(pickle_len, 0);
        file.read_exact(&mut header_pickle)?;

        // A Pickle buffer itself starts with a u32 payload length; the
        // serialized string length follows that prefix.
        let pickle_payload_size = u32::from_le_bytes(header_pickle[..4].try_into()?) as usize;
        let pickle_payload_end = 4usize
            .checked_add(pickle_payload_size)
            .ok_or_else(|| anyhow!("ASAR header pickle size overflow"))?;
        if pickle_payload_end > header_pickle.len() {
            bail!("ASAR header pickle payload exceeds its declared size");
        }
        let json_len = u32::from_le_bytes(header_pickle[4..8].try_into()?) as usize;
        let json_end = 8usize
            .checked_add(json_len)
            .ok_or_else(|| anyhow!("ASAR JSON header size overflow"))?;
        let json_bytes = header_pickle
            .get(8..json_end)
            .ok_or_else(|| anyhow!("ASAR JSON header exceeds its pickle"))?;
        if json_end > pickle_payload_end {
            bail!("ASAR JSON header exceeds the declared pickle payload");
        }
        let header: Value =
            serde_json::from_slice(json_bytes).context("invalid ASAR JSON header")?;
        let root_files = header
            .get("files")
            .and_then(Value::as_object)
            .ok_or_else(|| anyhow!("ASAR header has no files object"))?;

        let mut archive = Self {
            path,
            data_offset,
            files: HashMap::new(),
            directories: HashMap::new(),
        };
        archive.index_directory(root_files, "", file_len)?;
        Ok(archive)
    }

    fn index_directory(
        &mut self,
        children: &serde_json::Map<String, Value>,
        parent: &str,
        archive_len: u64,
    ) -> Result<()> {
        let parent_key = normalize_entry_path(parent);
        self.directories.entry(parent_key.clone()).or_default();

        for (name, node) in children {
            // ASAR names are path components. Reject traversal and separator
            // characters so unpacked files cannot escape their sidecar dir.
            if name.is_empty()
                || name == "."
                || name == ".."
                || name.contains('/')
                || name.contains('\\')
            {
                bail!("invalid ASAR path component {name:?}");
            }
            let relative_path = if parent.is_empty() {
                name.clone()
            } else {
                format!("{parent}/{name}")
            };
            let key = normalize_entry_path(&relative_path);

            if let Some(dir_children) = node.get("files").and_then(Value::as_object) {
                self.directories
                    .entry(parent_key.clone())
                    .or_default()
                    .insert(name.to_lowercase(), format!("{}/", name.to_lowercase()));
                self.index_directory(dir_children, &relative_path, archive_len)?;
                continue;
            }

            // Symlink records have a `link` field instead of file data. They
            // are not indexed as regular files by this filesystem.
            if node.get("link").is_some() {
                continue;
            }

            let Some(size) = node.get("size").and_then(Value::as_u64) else {
                continue;
            };
            let unpacked = node
                .get("unpacked")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let offset = if unpacked {
                0
            } else {
                let offset_text = node
                    .get("offset")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("ASAR file {relative_path} has no offset"))?;
                let offset = offset_text
                    .parse::<u64>()
                    .with_context(|| format!("invalid ASAR offset for {relative_path}"))?;
                let start = self
                    .data_offset
                    .checked_add(offset)
                    .ok_or_else(|| anyhow!("ASAR file offset overflow for {relative_path}"))?;
                let end = start
                    .checked_add(size)
                    .ok_or_else(|| anyhow!("ASAR file range overflow for {relative_path}"))?;
                if end > archive_len {
                    bail!("ASAR file {relative_path} extends beyond the archive");
                }
                offset
            };

            self.directories
                .entry(parent_key.clone())
                .or_default()
                .insert(name.to_lowercase(), name.to_lowercase());
            self.files.insert(
                key,
                FileEntry {
                    path: relative_path,
                    offset,
                    size,
                    unpacked,
                },
            );
        }
        Ok(())
    }

    fn has_file(&self, path: &str) -> bool {
        // ASAR's index defines virtual file existence. Unpacked payloads live
        // beside the archive and are checked when opening the stream.
        self.files.contains_key(&normalize_entry_path(path))
    }

    fn directory_exists(&self, path: &str) -> bool {
        self.directories.contains_key(&normalize_entry_path(path))
    }

    fn list_at(&self, path: &str) -> Vec<String> {
        self.directories
            .get(&normalize_entry_path(path))
            .map(|children| children.values().cloned().collect())
            .unwrap_or_default()
    }

    fn unpacked_path(&self, entry: &FileEntry) -> PathBuf {
        let mut sidecar = self.path.as_os_str().to_owned();
        sidecar.push(".unpacked");
        PathBuf::from(sidecar).join(Path::new(&entry.path))
    }

    fn open_entry(&self, entry: &FileEntry) -> io::Result<AsarReader> {
        if entry.unpacked {
            let file = File::open(self.unpacked_path(entry))?;
            let actual_len = file.metadata()?.len();
            if actual_len < entry.size {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unpacked ASAR file is shorter than its header size",
                ));
            }
            Ok(AsarReader::new(file, 0, entry.size))
        } else {
            let file = File::open(&self.path)?;
            let start = self.data_offset.checked_add(entry.offset).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "ASAR offset overflow")
            })?;
            Ok(AsarReader::new(file, start, entry.size))
        }
    }
}

fn normalize_entry_path(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/")
        .to_lowercase()
}

struct AsarReader {
    file: File,
    start: u64,
    len: u64,
    position: u64,
}

impl AsarReader {
    fn new(file: File, start: u64, len: u64) -> Self {
        Self {
            file,
            start,
            len,
            position: 0,
        }
    }
}

impl Read for AsarReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() || self.position >= self.len {
            return Ok(0);
        }
        let available = self.len - self.position;
        let count = buffer
            .len()
            .min(usize::try_from(available).unwrap_or(usize::MAX));
        let absolute = self
            .start
            .checked_add(self.position)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "ASAR position overflow"))?;
        self.file.seek(SeekFrom::Start(absolute))?;
        let read = self.file.read(&mut buffer[..count])?;
        self.position += read as u64;
        Ok(read)
    }
}

impl Seek for AsarReader {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let target = match position {
            SeekFrom::Start(offset) => i128::from(offset),
            SeekFrom::Current(offset) => i128::from(self.position) + i128::from(offset),
            SeekFrom::End(offset) => i128::from(self.len) + i128::from(offset),
        };
        if target < 0 || target > i128::from(u64::MAX) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek position in ASAR file",
            ));
        }
        self.position = target as u64;
        Ok(self.position)
    }
}

struct AsarFs {
    archives: Mutex<HashMap<String, Arc<Archive>>>,
}

impl AsarFs {
    fn new() -> Self {
        Self {
            archives: Mutex::new(HashMap::new()),
        }
    }

    fn archive(&self, path: &str) -> Option<Arc<Archive>> {
        let key = path.replace('\\', "/");
        if let Some(archive) = self.archives.lock().unwrap().get(&key) {
            return Some(archive.clone());
        }
        let path = Path::new(path);
        if !path.is_file() {
            return None;
        }
        match Archive::open(path) {
            Ok(archive) => {
                let archive = Arc::new(archive);
                self.archives.lock().unwrap().insert(key, archive.clone());
                Some(archive)
            }
            Err(error) => {
                log!("Failed to open ASAR {}: {error}", path.display());
                None
            }
        }
    }

    fn locate(&self, name: &str) -> Option<(Arc<Archive>, String)> {
        let name = name.replace('\\', "/");
        let parts: Vec<_> = name.split('/').filter(|part| !part.is_empty()).collect();
        for (index, component) in parts.iter().enumerate() {
            if !component.to_ascii_lowercase().ends_with(".asar") {
                continue;
            }
            let archive_path = parts[..=index].join("/");
            let entry_path = parts[index + 1..].join("/");
            if let Some(archive) = self.archive(&archive_path) {
                return Some((archive, entry_path));
            }
        }
        None
    }
}

impl TVPStorageMedia for AsarFs {
    fn get_name(&mut self, name: &mut ttstr) {
        name.assign("asar");
    }

    // Kirikiri's auto-path table matches lister entries by their exact key, so
    // normalize both queried paths and enumerated names the same way.
    fn normalize_domain_name(&mut self, name: &mut ttstr) {
        name.to_lower_case();
    }

    fn normalize_path_name(&mut self, name: &mut ttstr) {
        name.to_lower_case();
    }

    fn check_existent_storage(&mut self, name: &ttstr) -> bool {
        let name = name.to_string();
        let Some((archive, entry_path)) = self.locate(&name) else {
            return false;
        };
        if entry_path.is_empty() {
            return true;
        }
        archive.has_file(&entry_path) || archive.directory_exists(&entry_path)
    }

    fn open(&mut self, name: &ttstr, flags: tjs_uint32) -> *mut tTJSBinaryStream {
        if flags != TJS_BS_READ {
            return ptr::null_mut();
        }
        let name = name.to_string();
        let Some((archive, entry_path)) = self.locate(&name) else {
            return ptr::null_mut();
        };
        let Some(entry) = archive.files.get(&normalize_entry_path(&entry_path)) else {
            return ptr::null_mut();
        };
        let reader = match archive.open_entry(entry) {
            Ok(reader) => reader,
            Err(error) => {
                log!("Failed to open ASAR entry {entry_path}: {error}");
                return ptr::null_mut();
            }
        };
        let stream = com::create_istream_read_seek(reader);
        let binary_stream = unsafe { TVPCreateBinaryStreamAdapter(stream) };
        unsafe { (*stream).release() };
        binary_stream
    }

    fn get_list_at(&mut self, name: &ttstr, lister: *mut iTVPStorageLister) {
        if lister.is_null() || unsafe { (*lister).vtable_ }.is_null() {
            return;
        }
        let name = name.to_string();
        let Some((archive, entry_path)) = self.locate(&name) else {
            return;
        };
        for entry in archive.list_at(&entry_path) {
            let entry = entry.trim_start_matches(['/', '\\']);
            if entry.is_empty() {
                continue;
            }
            let entry = entry.to_lowercase();
            let entry = ttstr::from(entry.as_str());
            unsafe { (*lister).add(&entry) };
        }
    }

    fn get_locally_accessible_name(name: &mut ttstr) {
        name.assign("");
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system-unwind" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let fs = create_tvp_storage_media(AsarFs::new());
    unsafe {
        FS = fs;
        TVPRegisterStorageMedia(fs);
        GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount;
    }
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system-unwind" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[asar] Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unsafe {
        TVPUnregisterStorageMedia(FS);
        (*FS).release();
        FS = ptr::null_mut();
        TVPUninitImportStub();
    }
    0
}
