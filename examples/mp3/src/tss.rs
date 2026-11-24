//! COM implementation of the TVP Sound System module and wave decoder.

#![allow(non_snake_case, non_camel_case_types)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

use krkrz_plugin_base::tp_stub;
use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG, E_NOTIMPL, E_POINTER, HWND, S_FALSE, S_OK};
use windows::Win32::System::Com::IStream;
use windows::core::{ComObject, HRESULT, IUnknown, IUnknown_Vtbl, Interface, implement, interface};

use crate::decoder::{AudioFormat, Mp3Playback};

static LIVE_COM_OBJECTS: AtomicUsize = AtomicUsize::new(0);
static GLOBAL_REF_COUNT_AT_LINK: AtomicI32 = AtomicI32::new(0);

#[repr(C)]
#[derive(Clone, Copy, Default)]
#[allow(non_snake_case)]
struct TSSWaveFormat {
    dwSamplesPerSec: u32,
    dwChannels: u32,
    dwBitsPerSample: u32,
    dwSeekable: u32,
    ui64TotalSamples: u64,
    dwTotalTime: u32,
    dwReserved1: i32,
    dwReserved2: i32,
}

#[interface("7dd61993-615d-481d-b060-a9fd48394430")]
unsafe trait ITSSStorageProvider: IUnknown {
    fn GetStreamForRead(&self, url: *mut u16, stream: *mut *mut c_void) -> HRESULT;
    fn GetStreamForWrite(&self, url: *mut u16, stream: *mut *mut c_void) -> HRESULT;
    fn GetStreamForUpdate(&self, url: *mut u16, stream: *mut *mut c_void) -> HRESULT;
}

#[interface("a938d1a5-2980-498b-b051-99931d41895d")]
unsafe trait ITSSModule: IUnknown {
    fn GetModuleCopyright(&self, buffer: *mut u16, buflen: u32) -> HRESULT;
    fn GetModuleDescription(&self, buffer: *mut u16, buflen: u32) -> HRESULT;
    fn GetSupportExts(
        &self,
        index: u32,
        mediashortname: *mut u16,
        buffer: *mut u16,
        buflen: u32,
    ) -> HRESULT;
    fn GetMediaInfo(&self, url: *mut u16, info: *mut *mut c_void) -> HRESULT;
    fn GetMediaSupport(&self, url: *mut u16) -> HRESULT;
    fn GetMediaInstance(&self, url: *mut u16, instance: *mut *mut c_void) -> HRESULT;
}

#[interface("313864e2-910e-496f-8a6d-43465c105b58")]
unsafe trait ITSSWaveDecoder: IUnknown {
    fn GetFormat(&self, format: *mut TSSWaveFormat) -> HRESULT;
    fn Render(
        &self,
        buffer: *mut c_void,
        bufsamplelen: u32,
        rendered: *mut u32,
        status: *mut u32,
    ) -> HRESULT;
    fn SetPosition(&self, samplepos: u64) -> HRESULT;
}

#[implement(ITSSModule)]
struct Mp3Module {
    storage_provider: ITSSStorageProvider,
}

impl Drop for Mp3Module {
    fn drop(&mut self) {
        LIVE_COM_OBJECTS.fetch_sub(1, Ordering::AcqRel);
    }
}

impl ITSSModule_Impl for Mp3Module_Impl {
    unsafe fn GetModuleCopyright(&self, buffer: *mut u16, buflen: u32) -> HRESULT {
        copy_wide_string(buffer, buflen, "MP3 decoder for TVP Sound System (puremp3)")
    }

    unsafe fn GetModuleDescription(&self, buffer: *mut u16, buflen: u32) -> HRESULT {
        copy_wide_string(buffer, buflen, "MPEG Layer III audio (*.mp3) decoder")
    }

    unsafe fn GetSupportExts(
        &self,
        index: u32,
        mediashortname: *mut u16,
        buffer: *mut u16,
        buflen: u32,
    ) -> HRESULT {
        if index != 0 {
            return S_FALSE;
        }
        // ITSSModule does not provide a separate length for mediashortname.
        let short_name = copy_wide_string(mediashortname, 11, "MPEG Audio");
        if short_name.0 < 0 {
            return short_name;
        }
        copy_wide_string(buffer, buflen, ".mp3")
    }

    unsafe fn GetMediaInfo(&self, _url: *mut u16, info: *mut *mut c_void) -> HRESULT {
        if info.is_null() {
            return E_POINTER;
        }
        unsafe { info.write(ptr::null_mut()) };
        E_NOTIMPL
    }

    unsafe fn GetMediaSupport(&self, _url: *mut u16) -> HRESULT {
        // As in wuvorbis, actual support is established when the module opens
        // and parses the stream in GetMediaInstance.
        E_NOTIMPL
    }

    unsafe fn GetMediaInstance(&self, url: *mut u16, instance: *mut *mut c_void) -> HRESULT {
        if instance.is_null() {
            return E_POINTER;
        }
        unsafe { instance.write(ptr::null_mut()) };
        if url.is_null() {
            return E_POINTER;
        }

        let mut stream_raw = ptr::null_mut();
        let hr = unsafe { self.storage_provider.GetStreamForRead(url, &mut stream_raw) };
        if hr.0 < 0 {
            return hr;
        }
        if stream_raw.is_null() {
            return E_POINTER;
        }

        let stream_unknown = unsafe { IUnknown::from_raw(stream_raw) };
        let stream: IStream = match stream_unknown.cast() {
            Ok(stream) => stream,
            Err(error) => return error.code(),
        };
        drop(stream_unknown);

        let playback = match catch_unwind(AssertUnwindSafe(|| Mp3Playback::new(stream))) {
            Ok(Ok(playback)) => playback,
            Ok(Err(())) | Err(_) => return E_FAIL,
        };

        LIVE_COM_OBJECTS.fetch_add(1, Ordering::AcqRel);
        let decoder = ComObject::new(Mp3WaveDecoder {
            playback: RefCell::new(playback),
        });
        let decoder = decoder.into_interface::<ITSSWaveDecoder>();
        unsafe { instance.write(decoder.into_raw()) };
        S_OK
    }
}

#[implement(ITSSWaveDecoder)]
struct Mp3WaveDecoder {
    // TVP calls Render and SetPosition serially for a decoder instance. Keep
    // mutable stream state behind RefCell because COM methods take &self.
    playback: RefCell<Mp3Playback>,
}

impl Drop for Mp3WaveDecoder {
    fn drop(&mut self) {
        LIVE_COM_OBJECTS.fetch_sub(1, Ordering::AcqRel);
    }
}

impl ITSSWaveDecoder_Impl for Mp3WaveDecoder_Impl {
    unsafe fn GetFormat(&self, format: *mut TSSWaveFormat) -> HRESULT {
        if format.is_null() {
            return E_POINTER;
        }
        let audio = self.playback.borrow().format();
        unsafe { format.write(tss_format(audio)) };
        S_OK
    }

    unsafe fn Render(
        &self,
        buffer: *mut c_void,
        bufsamplelen: u32,
        rendered: *mut u32,
        status: *mut u32,
    ) -> HRESULT {
        if !rendered.is_null() {
            unsafe { rendered.write(0) };
        }
        if !status.is_null() {
            unsafe { status.write(0) };
        }

        let channels = self.playback.borrow().format().channels as usize;
        let sample_count = match (bufsamplelen as usize).checked_mul(channels) {
            Some(count) if count <= isize::MAX as usize / std::mem::size_of::<i16>() => count,
            _ => return E_INVALIDARG,
        };
        if sample_count != 0 && buffer.is_null() {
            return E_POINTER;
        }

        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut playback = self.playback.borrow_mut();
            let output = if sample_count == 0 {
                &mut []
            } else {
                unsafe { slice::from_raw_parts_mut(buffer.cast::<i16>(), sample_count) }
            };
            playback.render(output)
        }));

        let written = match result {
            Ok(Ok(written)) => written,
            Ok(Err(())) | Err(_) => return E_FAIL,
        };
        if !rendered.is_null() {
            unsafe { rendered.write(written as u32) };
        }
        if !status.is_null() {
            unsafe { status.write(u32::from(written == bufsamplelen as usize)) };
        }
        S_OK
    }

    unsafe fn SetPosition(&self, samplepos: u64) -> HRESULT {
        match catch_unwind(AssertUnwindSafe(|| {
            self.playback.borrow_mut().seek(samplepos)
        })) {
            Ok(Ok(())) => S_OK,
            Ok(Err(())) | Err(_) => E_FAIL,
        }
    }
}

fn tss_format(audio: AudioFormat) -> TSSWaveFormat {
    TSSWaveFormat {
        dwSamplesPerSec: audio.sample_rate,
        dwChannels: audio.channels,
        dwBitsPerSample: 16,
        // The decoder can seek by replaying frames from the start.
        dwSeekable: 2,
        // puremp3 does not expose a duration estimate for VBR streams.
        ui64TotalSamples: 0,
        dwTotalTime: 0,
        dwReserved1: 0,
        dwReserved2: 0,
    }
}

fn copy_wide_string(buffer: *mut u16, buflen: u32, value: &str) -> HRESULT {
    if buffer.is_null() {
        return E_POINTER;
    }
    if buflen == 0 {
        return E_INVALIDARG;
    }

    let capacity = buflen as usize;
    let mut count = 0;
    for (index, unit) in value.encode_utf16().take(capacity - 1).enumerate() {
        unsafe { buffer.add(index).write(unit) };
        count = index + 1;
    }
    unsafe { buffer.add(count).write(0) };
    S_OK
}

#[unsafe(export_name = "GetModuleInstance")]
unsafe extern "system" fn get_module_instance(
    output: *mut *mut c_void,
    storage_provider: *mut c_void,
    _config: *mut IStream,
    _main_window: HWND,
) -> HRESULT {
    if output.is_null() {
        return E_POINTER;
    }
    unsafe { output.write(ptr::null_mut()) };
    if storage_provider.is_null() {
        return E_POINTER;
    }

    let storage_provider = unsafe { ITSSStorageProvider::from_raw_borrowed(&storage_provider) };
    let Some(storage_provider) = storage_provider else {
        return E_POINTER;
    };

    LIVE_COM_OBJECTS.fetch_add(1, Ordering::AcqRel);
    let module = ComObject::new(Mp3Module {
        storage_provider: storage_provider.clone(),
    });
    let module = module.into_interface::<ITSSModule>();
    unsafe { output.write(module.into_raw()) };
    S_OK
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system-unwind" fn v2_link(exporter: *mut tp_stub::iTVPFunctionExporter) -> i32 {
    unsafe {
        tp_stub::TVPInitImportStub(exporter);
        GLOBAL_REF_COUNT_AT_LINK.store(tp_stub::TVPPluginGlobalRefCount, Ordering::Release);
    }
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system-unwind" fn v2_unlink() -> i32 {
    if LIVE_COM_OBJECTS.load(Ordering::Acquire) != 0
        || unsafe { tp_stub::TVPPluginGlobalRefCount }
            > GLOBAL_REF_COUNT_AT_LINK.load(Ordering::Acquire)
    {
        return tp_stub::TJS_E_FAIL;
    }
    unsafe { tp_stub::TVPUninitImportStub() };
    0
}
