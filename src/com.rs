//! Wrappers for COM type
use std::io::{Error, Read, Result, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};
use windows::Win32::Foundation::*;
use windows::Win32::System::Com;

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
