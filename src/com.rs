//! Wrappers for COM type
use std::cell::RefCell;
use std::io::{Error, Read, Result, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};
use windows::Win32::Foundation::*;
use windows::Win32::System::Com;
use windows_core::{IUnknownImpl, Interface, implement};

#[derive(Debug, Clone)]
/// Wrapper for [`IStream`](Com::IStream)
pub struct IStreamWrapper(pub Com::IStream);

impl IStreamWrapper {
    /// Try clone by using [`IStream::Clone`](Com::IStream::Clone)
    pub fn try_clone(&self) -> Result<Self> {
        unsafe { self.Clone() }
            .map(|s| Self(s))
            .map_err(|e| Error::from_raw_os_error(e.code().0))
    }

    /// Returns stream length
    pub fn stream_length(&mut self) -> Result<u64> {
        let mut stat = Com::STATSTG::default();
        unsafe { self.Stat(&mut stat, Com::STATFLAG_NONAME) }
            .map_err(|e| Error::from_raw_os_error(e.code().0))?;
        Ok(stat.cbSize)
    }
}

impl Deref for IStreamWrapper {
    type Target = Com::IStream;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IStreamWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Read for IStreamWrapper {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut readed = 0;
        let hr = unsafe {
            self.Read(
                buf.as_mut_ptr() as *mut _,
                buf.len() as u32,
                Some(&mut readed),
            )
        };
        if hr == S_OK || hr == S_FALSE {
            Ok(readed as usize)
        } else {
            Err(Error::from_raw_os_error(hr.0))
        }
    }
}

impl Write for IStreamWrapper {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut written = 0;
        let hr = unsafe {
            self.Write(
                buf.as_ptr() as *const _,
                buf.len() as u32,
                Some(&mut written),
            )
        };
        if hr == S_OK {
            Ok(written as usize)
        } else {
            Err(Error::from_raw_os_error(hr.0))
        }
    }

    fn flush(&mut self) -> Result<()> {
        unsafe { self.Commit(Com::STGC_DEFAULT) }.map_err(|e| Error::from_raw_os_error(e.code().0))
    }
}

impl Seek for IStreamWrapper {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let (origin, pos) = match pos {
            SeekFrom::Current(pos) => (Com::STREAM_SEEK_CUR, pos),
            SeekFrom::Start(pos) => (Com::STREAM_SEEK_SET, pos as i64),
            SeekFrom::End(pos) => (Com::STREAM_SEEK_END, pos),
        };
        let mut npos = 0;
        unsafe { self.Seek(pos, origin, Some(&mut npos)) }
            .map_err(|e| Error::from_raw_os_error(e.code().0))?;
        Ok(npos)
    }
}

pub trait IOOperation {
    /// Returns the reader for the current stream position.
    fn read<'a>(&'a mut self) -> Option<Box<dyn Read + 'a>> {
        None
    }

    /// Returns the seek handle for the current stream.
    fn seek<'a>(&'a mut self) -> Option<Box<dyn Seek + 'a>> {
        None
    }

    /// Returns the writer for the current stream position.
    fn write<'a>(&'a mut self) -> Option<Box<dyn Write + 'a>> {
        None
    }
}

#[implement(Com::IStream)]
struct IStreamAdapter<'a> {
    io: RefCell<Box<dyn IOOperation + 'a>>,
}

impl<'a> IStreamAdapter<'a> {
    fn error(error: Error) -> windows::core::HRESULT {
        error
            .raw_os_error()
            .filter(|code| *code > 0)
            .map(|code| windows_core::HRESULT::from_win32(code as u32))
            .unwrap_or(E_FAIL)
    }

    fn unsupported() -> windows::core::HRESULT {
        E_NOTIMPL
    }

    fn borrow_error() -> windows::core::HRESULT {
        E_FAIL
    }
}

impl<'a> Drop for IStreamAdapter<'a> {
    fn drop(&mut self) {
        unsafe {
            crate::tp_stub::TVPPluginGlobalRefCount -= 1;
        }
    }
}

impl<'a> Com::ISequentialStream_Impl for IStreamAdapter_Impl<'a> {
    fn Read(
        &self,
        pv: *mut std::ffi::c_void,
        cb: u32,
        pcbread: *mut u32,
    ) -> windows::core::HRESULT {
        if cb != 0 && pv.is_null() {
            return E_POINTER;
        }

        let mut s = match self.get_impl().io.try_borrow_mut() {
            Ok(io) => io,
            Err(_) => return IStreamAdapter::borrow_error(),
        };
        let reader = s.read();
        let Some(mut reader) = reader else {
            return IStreamAdapter::unsupported();
        };
        let buffer = if cb == 0 {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(pv as *mut u8, cb as usize) }
        };
        let result = reader.read(buffer).map_err(IStreamAdapter::error);
        match result {
            Ok(read) if read <= cb as usize => {
                if !pcbread.is_null() {
                    unsafe { *pcbread = read as u32 };
                }
                if read < cb as usize { S_FALSE } else { S_OK }
            }
            Ok(_) => E_FAIL,
            Err(error) => error,
        }
    }

    fn Write(
        &self,
        pv: *const std::ffi::c_void,
        cb: u32,
        pcbwritten: *mut u32,
    ) -> windows::core::HRESULT {
        if cb != 0 && pv.is_null() {
            return E_POINTER;
        }

        let mut s = match self.get_impl().io.try_borrow_mut() {
            Ok(io) => io,
            Err(_) => return IStreamAdapter::borrow_error(),
        };
        let writer = s.write();
        let Some(mut writer) = writer else {
            return IStreamAdapter::unsupported();
        };
        let buffer = if cb == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(pv as *const u8, cb as usize) }
        };
        let result = writer.write(buffer).map_err(IStreamAdapter::error);
        match result {
            Ok(written) if written <= cb as usize => {
                if !pcbwritten.is_null() {
                    unsafe { *pcbwritten = written as u32 };
                }
                S_OK
            }
            Ok(_) => E_FAIL,
            Err(error) => error,
        }
    }
}

impl<'a> Com::IStream_Impl for IStreamAdapter_Impl<'a> {
    fn Seek(
        &self,
        offset: i64,
        origin: Com::STREAM_SEEK,
        new_position: *mut u64,
    ) -> windows::core::Result<()> {
        let from = match origin {
            Com::STREAM_SEEK_SET => SeekFrom::Start(offset.try_into().map_err(|_| E_INVALIDARG)?),
            Com::STREAM_SEEK_CUR => SeekFrom::Current(offset),
            Com::STREAM_SEEK_END => SeekFrom::End(offset),
            _ => return Err(E_INVALIDARG.into()),
        };
        let mut s = self
            .get_impl()
            .io
            .try_borrow_mut()
            .map_err(|_| IStreamAdapter::borrow_error())?;
        let mut seeker = s.seek().ok_or(E_NOTIMPL)?;
        let position = seeker.seek(from).map_err(IStreamAdapter::error)?;
        if !new_position.is_null() {
            unsafe { *new_position = position };
        }
        Ok(())
    }

    fn SetSize(&self, _size: u64) -> windows::core::Result<()> {
        Err(IStreamAdapter::unsupported().into())
    }

    fn CopyTo(
        &self,
        _stream: windows::core::Ref<Com::IStream>,
        _size: u64,
        _read: *mut u64,
        _written: *mut u64,
    ) -> windows::core::Result<()> {
        Err(IStreamAdapter::unsupported().into())
    }

    fn Commit(&self, _flags: &Com::STGC) -> windows::core::Result<()> {
        let mut s = self
            .get_impl()
            .io
            .try_borrow_mut()
            .map_err(|_| IStreamAdapter::borrow_error())?;
        let mut writer = s.write().ok_or(E_NOTIMPL)?;
        writer.flush().map_err(IStreamAdapter::error)?;
        Ok(())
    }

    fn Revert(&self) -> windows::core::Result<()> {
        Err(IStreamAdapter::unsupported().into())
    }

    fn LockRegion(
        &self,
        _offset: u64,
        _size: u64,
        _lock_type: &Com::LOCKTYPE,
    ) -> windows::core::Result<()> {
        Err(IStreamAdapter::unsupported().into())
    }

    fn UnlockRegion(&self, _offset: u64, _size: u64, _lock_type: u32) -> windows::core::Result<()> {
        Err(IStreamAdapter::unsupported().into())
    }

    fn Stat(&self, stat: *mut Com::STATSTG, _flags: &Com::STATFLAG) -> windows::core::Result<()> {
        if stat.is_null() {
            return Err(E_POINTER.into());
        }

        let mut s = self
            .get_impl()
            .io
            .try_borrow_mut()
            .map_err(|_| IStreamAdapter::borrow_error())?;
        let mut seeker = s.seek().ok_or(E_NOTIMPL)?;
        let current = seeker.stream_position().map_err(IStreamAdapter::error)?;
        let size = seeker
            .seek(SeekFrom::End(0))
            .map_err(IStreamAdapter::error)?;
        seeker
            .seek(SeekFrom::Start(current))
            .map_err(IStreamAdapter::error)?;
        unsafe {
            *stat = Com::STATSTG::default();
            (*stat).r#type = Com::STGTY_STREAM.0 as u32;
            (*stat).cbSize = size;
        }
        Ok(())
    }

    fn Clone(&self) -> windows::core::Result<Com::IStream> {
        Err(IStreamAdapter::unsupported().into())
    }
}

/// Creates a TVP-compatible [`IStream`](crate::tp_stub::IStream) backed by `io`.
///
/// The returned pointer owns one COM reference. The caller must eventually call its
/// `Release` method; converting it to a Windows [`Com::IStream`] with `from_raw` transfers
/// that ownership to the Windows interface value.
///
/// `IOOperation::read`, `IOOperation::seek`, and `IOOperation::write` are called as needed by
/// the corresponding COM methods. Unsupported operations return `E_NOTIMPL`. The adapter does
/// not synchronize `io`; callers must not use the returned stream concurrently.
///
/// # Example
///
/// ```no_run
/// use krkrz_plugin_base::com::{create_istream, IOOperation};
/// use std::io::{Cursor, Read, Seek, Write};
///
/// struct MemoryStream(Cursor<Vec<u8>>);
///
/// impl IOOperation for MemoryStream {
///     fn read(&mut self) -> Option<Box<dyn Read>> { Some(Box::new(self.0.clone())) }
///     fn seek(&mut self) -> Option<Box<dyn Seek>> { Some(Box::new(self.0.clone())) }
///     fn write(&mut self) -> Option<Box<dyn Write>> { Some(Box::new(self.0.clone())) }
/// }
///
/// let stream = create_istream(MemoryStream(Cursor::new(Vec::new())));
/// let stream = unsafe { windows::Win32::System::Com::IStream::from_raw(stream as *mut _) };
/// drop(stream);
/// ```
pub fn create_istream<T: IOOperation>(io: T) -> *mut crate::tp_stub::IStream {
    unsafe {
        crate::tp_stub::TVPPluginGlobalRefCount += 1;
    }
    let stream = windows::core::ComObject::new(IStreamAdapter {
        io: RefCell::new(Box::new(io)),
    })
    .into_interface::<Com::IStream>()
    .into_raw();
    stream as *mut crate::tp_stub::IStream
}

/// Creates a TVP-compatible [`IStream`](crate::tp_stub::IStream) backed by a reader.
///
/// The returned stream supports read operations only. See [`create_istream`] for pointer
/// ownership and concurrency details.
pub fn create_istream_read<R: Read>(io: R) -> *mut crate::tp_stub::IStream {
    struct Wrapper<R: Read> {
        io: R,
    }
    impl<R: Read> IOOperation for Wrapper<R> {
        fn read<'a>(&'a mut self) -> Option<Box<dyn Read + 'a>> {
            Some(Box::new(&mut self.io))
        }
    }
    create_istream(Wrapper { io })
}

/// Creates a TVP-compatible [`IStream`](crate::tp_stub::IStream) backed by a writer.
///
/// The returned stream supports write operations only. See [`create_istream`] for pointer
/// ownership and concurrency details.
pub fn create_istream_write<W: Write>(io: W) -> *mut crate::tp_stub::IStream {
    struct Wrapper<W: Write> {
        io: W,
    }
    impl<W: Write> IOOperation for Wrapper<W> {
        fn write<'a>(&'a mut self) -> Option<Box<dyn Write + 'a>> {
            Some(Box::new(&mut self.io))
        }
    }
    create_istream(Wrapper { io })
}

/// Creates a TVP-compatible [`IStream`](crate::tp_stub::IStream) backed by a reader and seeker.
///
/// The returned stream supports read and seek operations. See [`create_istream`] for pointer
/// ownership and concurrency details.
pub fn create_istream_read_seek<R: Read + Seek>(io: R) -> *mut crate::tp_stub::IStream {
    struct Wrapper<R: Read + Seek> {
        io: R,
    }
    impl<R: Read + Seek> IOOperation for Wrapper<R> {
        fn read<'a>(&'a mut self) -> Option<Box<dyn Read + 'a>> {
            Some(Box::new(&mut self.io))
        }
        fn seek<'a>(&'a mut self) -> Option<Box<dyn Seek + 'a>> {
            Some(Box::new(&mut self.io))
        }
    }
    create_istream(Wrapper { io })
}

/// Creates a TVP-compatible [`IStream`](crate::tp_stub::IStream) backed by a writer and seeker.
///
/// The returned stream supports write and seek operations. See [`create_istream`] for pointer
/// ownership and concurrency details.
pub fn create_istream_write_seek<W: Write + Seek>(io: W) -> *mut crate::tp_stub::IStream {
    struct Wrapper<W: Write + Seek> {
        io: W,
    }
    impl<W: Write + Seek> IOOperation for Wrapper<W> {
        fn write<'a>(&'a mut self) -> Option<Box<dyn Write + 'a>> {
            Some(Box::new(&mut self.io))
        }
        fn seek<'a>(&'a mut self) -> Option<Box<dyn Seek + 'a>> {
            Some(Box::new(&mut self.io))
        }
    }
    create_istream(Wrapper { io })
}

/// Creates a TVP-compatible [`IStream`](crate::tp_stub::IStream) backed by a readable, writable,
/// and seekable object.
///
/// The returned stream supports read, write, and seek operations. See [`create_istream`] for
/// pointer ownership and concurrency details.
pub fn create_istream_all<A: Read + Seek + Write>(io: A) -> *mut crate::tp_stub::IStream {
    struct Wrapper<A: Read + Seek + Write> {
        io: A,
    }
    impl<A: Read + Seek + Write> IOOperation for Wrapper<A> {
        fn read<'a>(&'a mut self) -> Option<Box<dyn Read + 'a>> {
            Some(Box::new(&mut self.io))
        }
        fn seek<'a>(&'a mut self) -> Option<Box<dyn Seek + 'a>> {
            Some(Box::new(&mut self.io))
        }
        fn write<'a>(&'a mut self) -> Option<Box<dyn Write + 'a>> {
            Some(Box::new(&mut self.io))
        }
    }
    create_istream(Wrapper { io })
}
