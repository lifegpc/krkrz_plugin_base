use image::codecs::avif::AvifDecoder;
use image::{ColorType, ImageDecoder};
use krkrz_plugin_base::{com::IStreamWrapper, tp_stub::*, *};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use windows::Win32::System::Com::IStream as ComIStream;
use windows::core::Interface;

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;

unsafe extern "C-unwind" fn load_avif(
    _formatdata: *mut std::ffi::c_void,
    callbackdata: *mut std::ffi::c_void,
    sizecallback: tTVPGraphicSizeCallback,
    scanlinecallback: tTVPGraphicScanLineCallback,
    _metainfopushcallback: tTVPMetaInfoPushCallback,
    src: *mut IStream,
    _keyidx: tjs_int32,
    mode: tTVPGraphicLoadMode,
) {
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        load_image(callbackdata, sizecallback, scanlinecallback, src, mode)
    }));

    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => throw_exception_message!("[avif] {error}"),
        Err(_) => throw_exception_message!("[avif] Unexpected panic while decoding image"),
    }
}

unsafe fn load_image(
    callbackdata: *mut std::ffi::c_void,
    sizecallback: tTVPGraphicSizeCallback,
    scanlinecallback: tTVPGraphicScanLineCallback,
    src: *mut IStream,
    mode: tTVPGraphicLoadMode,
) -> Result<(), String> {
    let sizecallback = sizecallback.ok_or("TVP size callback is unavailable")?;
    let scanlinecallback = scanlinecallback.ok_or("TVP scanline callback is unavailable")?;
    let (width, height, rgba) = unsafe { decode_avif(src)? };

    unsafe { sizecallback(callbackdata, width, height) };

    let width = width as usize;
    let row_bytes = width
        .checked_mul(4)
        .ok_or("Image row size overflows address space")?;
    let mut bgra_row = if mode == tTVPGraphicLoadMode_glmNormal {
        Vec::new()
    } else {
        allocate_bytes(row_bytes)?
    };

    for y in 0..height as usize {
        let row_start = y * row_bytes;
        let rgba_row = &rgba[row_start..row_start + row_bytes];
        let destination = unsafe { scanlinecallback(callbackdata, y as tjs_int) };
        if destination.is_null() {
            break;
        }

        if mode == tTVPGraphicLoadMode_glmNormal {
            let destination =
                unsafe { std::slice::from_raw_parts_mut(destination.cast::<u8>(), row_bytes) };
            for (dst, src) in destination
                .chunks_exact_mut(4)
                .zip(rgba_row.chunks_exact(4))
            {
                dst.copy_from_slice(&[src[2], src[1], src[0], src[3]]);
            }
        } else {
            for (dst, src) in bgra_row.chunks_exact_mut(4).zip(rgba_row.chunks_exact(4)) {
                dst.copy_from_slice(&[src[2], src[1], src[0], src[3]]);
            }
            unsafe {
                TVPBLConvert32BitTo8Bit(
                    destination.cast::<tjs_uint8>(),
                    bgra_row.as_ptr().cast::<tjs_uint32>(),
                    width as tjs_int,
                );
            }
        }

        unsafe { scanlinecallback(callbackdata, -1) };
    }

    Ok(())
}

unsafe fn decode_avif(src: *mut IStream) -> Result<(u32, u32, Vec<u8>), String> {
    if src.is_null() {
        return Err("Source stream is null".to_owned());
    }

    // The stream belongs to TVP. Add a COM reference before wrapping it in an
    // owning Windows interface, which releases that extra reference on drop.
    let mut source = unsafe {
        (*src).add_ref();
        IStreamWrapper(ComIStream::from_raw(src.cast()))
    };
    source
        .seek(SeekFrom::Start(0))
        .map_err(|error| format!("Could not seek input stream: {error}"))?;
    let mut encoded = Vec::new();
    source
        .read_to_end(&mut encoded)
        .map_err(|error| format!("Could not read input stream: {error}"))?;

    let decoder = AvifDecoder::new(Cursor::new(encoded))
        .map_err(|error| format!("Could not decode AVIF: {error}"))?;
    let (width, height) = decoder.dimensions();
    if width == 0 || height == 0 {
        return Err("Image dimensions cannot be zero".to_owned());
    }
    if width > tjs_int::MAX as u32 || height > tjs_int::MAX as u32 {
        return Err("Image dimensions exceed the TVP limit".to_owned());
    }

    let pixel_count = (width as usize)
        .checked_mul(height as usize)
        .ok_or("Image dimensions overflow address space")?;
    let rgba8_len = pixel_count
        .checked_mul(4)
        .ok_or("Decoded image size overflows address space")?;
    let color_type = decoder.color_type();
    let decoded_len = usize::try_from(decoder.total_bytes())
        .map_err(|_| "Decoded image is too large for this process")?;
    let mut decoded = allocate_bytes(decoded_len)?;
    decoder
        .read_image(&mut decoded)
        .map_err(|error| format!("Could not decode AVIF pixels: {error}"))?;

    let rgba = match color_type {
        ColorType::Rgba8 if decoded.len() == rgba8_len => decoded,
        ColorType::Rgba16 if decoded.len() == rgba8_len.saturating_mul(2) => {
            let mut rgba8 = allocate_bytes(rgba8_len)?;
            for (dst, src) in rgba8.iter_mut().zip(decoded.chunks_exact(2)) {
                let value = u16::from_ne_bytes([src[0], src[1]]) as u32;
                *dst = ((value * 255 + 32_767) / 65_535) as u8;
            }
            rgba8
        }
        _ => return Err("AVIF decoder returned an unexpected pixel format".to_owned()),
    };

    Ok((width, height, rgba))
}

fn allocate_bytes(size: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|error| format!("Could not allocate image buffer: {error}"))?;
    bytes.resize(size, 0);
    Ok(bytes)
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system-unwind" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };

    let extension = ttstr::from(".avif");
    unsafe {
        TVPRegisterGraphicLoadingHandler(
            &extension,
            Some(load_avif),
            None,
            None,
            None,
            ptr::null_mut(),
        );
        GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount;
    }
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system-unwind" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[avif] Cannot unlink while plugin references are still active");
        return TJS_E_FAIL;
    }

    let extension = ttstr::from(".avif");
    unsafe {
        TVPUnregisterGraphicLoadingHandler(
            &extension,
            Some(load_avif),
            None,
            None,
            None,
            ptr::null_mut(),
        );
        TVPUninitImportStub();
    }
    0
}
