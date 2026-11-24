//! DirectShow COM surface built from the generated `windows` interfaces.

use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use windows::Win32::Foundation::{
    E_FAIL, E_INVALIDARG, E_NOTIMPL, E_OUTOFMEMORY, E_POINTER, RECT, S_FALSE, S_OK,
};
use windows::Win32::Graphics::Gdi::BITMAPINFOHEADER;
use windows::Win32::Media::DirectShow::*;
use windows::Win32::Media::IReferenceClock;
use windows::Win32::Media::MediaFoundation::{
    AM_MEDIA_TYPE, FORMAT_VideoInfo, FORMAT_WaveFormatEx, MEDIASUBTYPE_PCM, MEDIASUBTYPE_RGB32,
    MEDIATYPE_Audio, MEDIATYPE_Stream, MEDIATYPE_Video,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, CoCreateInstance, CoTaskMemAlloc, CoTaskMemFree, IPersist_Impl,
};
use windows::core::{
    BOOL, ComObject, Error, GUID, HRESULT, IUnknown, IUnknownImpl, Interface, PCWSTR, PWSTR, Ref,
    implement,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum FilterKind {
    WebmSplitter,
    Vp9Decoder,
    OpusDecoder,
    #[allow(dead_code)]
    DisabledAudio,
}

#[derive(Clone, Copy)]
enum FormatKind {
    None,
    Vp9 { width: u32, height: u32 },
    Rgb32 { width: u32, height: u32 },
    Pcm16Stereo48k,
}

#[derive(Clone, Copy)]
struct MediaSpec {
    major: GUID,
    subtype: GUID,
    format: FormatKind,
}

use windows::Win32::Media::MediaFoundation::VIDEOINFOHEADER as VideoInfoHeader;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct WaveFormatEx {
    format_tag: u16,
    channels: u16,
    samples_per_sec: u32,
    avg_bytes_per_sec: u32,
    block_align: u16,
    bits_per_sample: u16,
    extra_size: u16,
}

#[derive(Clone)]
struct PinSpec {
    id: &'static str,
    name: &'static str,
    direction: PIN_DIRECTION,
    media: Vec<MediaSpec>,
}

struct PinState {
    spec: PinSpec,
    owner: Weak<FilterState>,
    peer: Mutex<Option<IPin>>,
    connection_type: Mutex<Option<MediaSpec>>,
    allocator: Mutex<Option<IMemAllocator>>,
    received: Mutex<Vec<u8>>,
    received_samples: AtomicUsize,
    delivered_samples: AtomicUsize,
}

struct FilterState {
    kind: FilterKind,
    name: &'static str,
    class_id: GUID,
    state: Mutex<FILTER_STATE>,
    graph: Mutex<Option<IFilterGraph>>,
    pins: Vec<Arc<PinState>>,
    pin_interfaces: Mutex<Vec<IPin>>,
    video_dimensions: Mutex<Option<(u32, u32)>>,
    pull_source: AtomicBool,
    demux_started: AtomicBool,
    stop_requested: AtomicBool,
    #[cfg(feature = "vp9")]
    vp9: Mutex<Option<crate::vp9::Vp9Decoder>>,
    #[cfg(feature = "opus")]
    opus: Mutex<Option<crate::opus::OpusDecoder>>,
}

// DirectShow delivers samples from filter worker threads. All mutable pin and
// filter state is synchronized; the demux worker does not access `graph`.
unsafe impl Send for PinState {}
unsafe impl Sync for PinState {}
unsafe impl Send for FilterState {}
unsafe impl Sync for FilterState {}

fn fourcc(code: &[u8; 4]) -> GUID {
    GUID::from_u128(
        ((u32::from_le_bytes(*code) as u128) << 96)
            | (0x0010u128 << 64)
            | 0x8000_00aa_0038_9b71u128,
    )
}

fn media(major: GUID, subtype: GUID) -> MediaSpec {
    MediaSpec {
        major,
        subtype,
        format: FormatKind::None,
    }
}

fn output_media(major: GUID, subtype: GUID, format: FormatKind) -> MediaSpec {
    MediaSpec {
        major,
        subtype,
        format,
    }
}

fn resolved_media_types(pin: &PinState) -> Vec<MediaSpec> {
    let Some(owner) = pin.owner.upgrade() else {
        return pin.spec.media.clone();
    };
    let Some((width, height)) = *owner.video_dimensions.lock().unwrap() else {
        return pin.spec.media.clone();
    };
    pin.spec
        .media
        .iter()
        .map(|spec| {
            let format = match (owner.kind, pin.spec.id, spec.format) {
                (FilterKind::WebmSplitter, "Video", FormatKind::Vp9 { .. }) => {
                    FormatKind::Vp9 { width, height }
                }
                (FilterKind::Vp9Decoder, "Output", FormatKind::Rgb32 { .. }) => {
                    FormatKind::Rgb32 { width, height }
                }
                _ => spec.format,
            };
            MediaSpec { format, ..*spec }
        })
        .collect()
}

fn filter_state(kind: FilterKind) -> Arc<FilterState> {
    crate::videos_log!("[videos] building {:?} filter state", kind);
    let (name, class_id, mut specs) = match kind {
        FilterKind::WebmSplitter => (
            "Rust WebM Splitter",
            GUID::from_u128(0x6ed33d7a_2b16_4f9d_8f13_fbd3cf8c3201),
            vec![
                PinSpec {
                    id: "Input",
                    name: "WebM input",
                    direction: PINDIR_INPUT,
                    media: vec![media(MEDIATYPE_Stream, GUID::zeroed())],
                },
                PinSpec {
                    id: "Video",
                    name: "VP9 output",
                    direction: PINDIR_OUTPUT,
                    media: vec![output_media(
                        MEDIATYPE_Video,
                        fourcc(b"VP90"),
                        FormatKind::Vp9 {
                            width: 1,
                            height: 1,
                        },
                    )],
                },
                PinSpec {
                    id: "Audio",
                    name: "Opus output",
                    direction: PINDIR_OUTPUT,
                    media: vec![media(MEDIATYPE_Audio, fourcc(b"OPUS"))],
                },
            ],
        ),
        FilterKind::Vp9Decoder => (
            "Rust VP9 Decoder",
            GUID::from_u128(0x6ed33d7a_2b16_4f9d_8f13_fbd3cf8c3202),
            vec![
                PinSpec {
                    id: "Input",
                    name: "VP9 input",
                    direction: PINDIR_INPUT,
                    media: vec![media(MEDIATYPE_Video, fourcc(b"VP90"))],
                },
                PinSpec {
                    id: "Output",
                    name: "RGB32 output",
                    direction: PINDIR_OUTPUT,
                    media: vec![output_media(
                        MEDIATYPE_Video,
                        MEDIASUBTYPE_RGB32,
                        FormatKind::Rgb32 {
                            width: 1,
                            height: 1,
                        },
                    )],
                },
            ],
        ),
        FilterKind::OpusDecoder => (
            "Rust Opus Decoder",
            GUID::from_u128(0x6ed33d7a_2b16_4f9d_8f13_fbd3cf8c3203),
            vec![
                PinSpec {
                    id: "Input",
                    name: "Opus input",
                    direction: PINDIR_INPUT,
                    media: vec![media(MEDIATYPE_Audio, fourcc(b"OPUS"))],
                },
                PinSpec {
                    id: "Output",
                    name: "PCM output",
                    direction: PINDIR_OUTPUT,
                    media: vec![output_media(
                        MEDIATYPE_Audio,
                        MEDIASUBTYPE_PCM,
                        FormatKind::Pcm16Stereo48k,
                    )],
                },
            ],
        ),
        FilterKind::DisabledAudio => (
            "Disabled Opus Decoder",
            GUID::from_u128(0x6ed33d7a_2b16_4f9d_8f13_fbd3cf8c3204),
            Vec::new(),
        ),
    };
    if matches!(kind, FilterKind::WebmSplitter) {
        specs.retain(|pin| match pin.id {
            "Video" => cfg!(feature = "vp9"),
            "Audio" => cfg!(feature = "opus"),
            _ => true,
        });
    }
    let state = Arc::new_cyclic(|owner| FilterState {
        kind,
        name,
        class_id,
        state: Mutex::new(State_Stopped),
        graph: Mutex::new(None),
        pins: specs
            .into_iter()
            .map(|spec| {
                Arc::new(PinState {
                    spec,
                    owner: owner.clone(),
                    peer: Mutex::new(None),
                    connection_type: Mutex::new(None),
                    allocator: Mutex::new(None),
                    received: Mutex::new(Vec::new()),
                    received_samples: AtomicUsize::new(0),
                    delivered_samples: AtomicUsize::new(0),
                })
            })
            .collect(),
        pin_interfaces: Mutex::new(Vec::new()),
        video_dimensions: Mutex::new(None),
        pull_source: AtomicBool::new(false),
        demux_started: AtomicBool::new(false),
        stop_requested: AtomicBool::new(false),
        #[cfg(feature = "vp9")]
        vp9: Mutex::new(None),
        #[cfg(feature = "opus")]
        opus: Mutex::new(None),
    });
    crate::videos_log!(
        "[videos] {:?} filter state ready ({} pins)",
        kind,
        state.pins.len()
    );
    state
}

const CLSID_MEMORY_ALLOCATOR: GUID = GUID::from_u128(0x1e651cc0_b199_11d0_8212_00c04fc32c45);

fn input_allocator(state: &PinState) -> windows::core::Result<IMemAllocator> {
    if let Some(allocator) = state.allocator.lock().unwrap().clone() {
        return Ok(allocator);
    }
    let allocator: IMemAllocator = unsafe {
        CoCreateInstance(
            &CLSID_MEMORY_ALLOCATOR,
            None::<&IUnknown>,
            CLSCTX_INPROC_SERVER,
        )?
    };
    *state.allocator.lock().unwrap() = Some(allocator.clone());
    Ok(allocator)
}

fn connect_allocator(output: &PinState, receive: &IPin) -> windows::core::Result<()> {
    crate::videos_log!(
        "[videos] negotiating allocator for output pin {}",
        output.spec.id
    );
    let input = receive.cast::<IMemInputPin>()?;
    let allocator =
        unsafe { input.GetAllocator() }.or_else(|_| input_allocator_from_peer(&input))?;
    // krkrz's vomLayer renderer only accepts its own allocator with exactly
    // one buffer whose size matches the negotiated RGB32 dimensions.
    let media_spec = resolved_media_types(output)
        .into_iter()
        .next()
        .ok_or_else(|| Error::from_hresult(VFW_E_NO_ACCEPTABLE_TYPES))?;
    let (buffer_count, buffer_size) = match media_spec.format {
        FormatKind::Rgb32 { width, height } => (
            1,
            width
                .checked_mul(height)
                .and_then(|pixels| pixels.checked_mul(4))
                .and_then(|bytes| i32::try_from(bytes).ok())
                .ok_or_else(|| Error::from_hresult(E_INVALIDARG))?,
        ),
        FormatKind::Vp9 { .. } => (4, 16 * 1024 * 1024),
        FormatKind::Pcm16Stereo48k => (4, 192_000),
        FormatKind::None => (4, 16 * 1024 * 1024),
    };
    let request = ALLOCATOR_PROPERTIES {
        cBuffers: buffer_count,
        cbBuffer: buffer_size,
        cbAlign: 1,
        cbPrefix: 0,
    };
    unsafe {
        allocator.SetProperties(&request)?;
        allocator.Commit()?;
        input.NotifyAllocator(&allocator, false)?;
    }
    crate::videos_log!(
        "[videos] allocator ready for {}: buffers={}, bytes={}",
        output.spec.id,
        buffer_count,
        buffer_size
    );
    *output.allocator.lock().unwrap() = Some(allocator);
    Ok(())
}

fn input_allocator_from_peer(input: &IMemInputPin) -> windows::core::Result<IMemAllocator> {
    // DirectShow's standard in-proc memory allocator.
    unsafe {
        let allocator: IMemAllocator = CoCreateInstance(
            &CLSID_MEMORY_ALLOCATOR,
            None::<&IUnknown>,
            CLSCTX_INPROC_SERVER,
        )?;
        input.NotifyAllocator(&allocator, false)?;
        Ok(allocator)
    }
}

fn deliver_sample(
    output: &PinState,
    bytes: &[u8],
    time: Option<(i64, i64)>,
    sync_point: bool,
) -> windows::core::Result<()> {
    #[cfg(feature = "log")]
    let delivery_started = std::time::Instant::now();
    let peer = output
        .peer
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| Error::from_hresult(VFW_E_NOT_CONNECTED))?;
    let allocator = output
        .allocator
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| Error::from_hresult(VFW_E_NOT_CONNECTED))?;
    let mut sample = None;
    unsafe {
        allocator.GetBuffer(&mut sample, None, None, 0)?;
    }
    let sample = sample.ok_or_else(|| Error::from_hresult(E_FAIL))?;
    if bytes.len() > unsafe { sample.GetSize() } as usize {
        return Err(Error::from_hresult(VFW_E_BUFFER_OVERFLOW));
    }
    let destination = unsafe { sample.GetPointer()? };
    if !bytes.is_empty() {
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len());
        }
    }
    unsafe {
        sample.SetActualDataLength(bytes.len() as i32)?;
        sample.SetSyncPoint(sync_point)?;
        if let Some((start, stop)) = time {
            sample.SetTime(Some(&start), Some(&stop))?;
        }
        peer.cast::<IMemInputPin>()?.Receive(&sample)?;
    }
    let number = output.delivered_samples.fetch_add(1, Ordering::Relaxed) + 1;
    if number == 1 || number % 300 == 0 {
        crate::videos_log!(
            "[videos] delivered {} sample {} ({} bytes) in {} ms",
            output.spec.id,
            number,
            bytes.len(),
            {
                #[cfg(feature = "log")]
                {
                    delivery_started.elapsed().as_millis()
                }
                #[cfg(not(feature = "log"))]
                {
                    0
                }
            }
        );
    }
    Ok(())
}

fn send_eos(state: &FilterState) {
    for pin in &state.pins {
        if pin.spec.direction == PINDIR_OUTPUT {
            if let Some(peer) = pin.peer.lock().unwrap().clone() {
                unsafe {
                    let _ = peer.EndOfStream();
                }
            }
        }
    }
}

fn process_webm(state: &FilterState) -> windows::core::Result<()> {
    #[cfg(feature = "webm")]
    {
        let input = state
            .pins
            .iter()
            .find(|pin| pin.spec.id == "Input")
            .ok_or_else(|| Error::from_hresult(E_FAIL))?;
        let bytes = std::mem::take(&mut *input.received.lock().unwrap());
        crate::videos_log!("[videos] WebM demux starting with {} source bytes", bytes.len());
        let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::webm::WebmFile::parse(&bytes)
        }));
        let file = match parsed {
            Ok(Ok(file)) => file,
            Ok(Err(_error)) => {
                crate::videos_log!("[videos] WebM parse error: {_error:?}");
                return Err(Error::from_hresult(E_FAIL));
            }
            Err(_) => {
                crate::videos_log!("[videos] panic while parsing WebM");
                return Err(Error::from_hresult(E_FAIL));
            }
        };
        crate::videos_log!(
            "[videos] WebM parsed: {} tracks, {} packets",
            file.tracks.len(),
            file.packets.len()
        );
        for _track in &file.tracks {
            crate::videos_log!(
                "[videos] track {}: {} {:?}x{:?} channels={:?}",
                _track.number,
                _track.codec_id,
                _track.width,
                _track.height,
                _track.channels
            );
        }
        let tracks = file
            .tracks
            .iter()
            .map(|track| {
                (
                    track.number,
                    (track.codec_id.as_str(), track.default_duration_ns),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();
        let mut video_packets = Vec::new();
        let mut audio_packets = Vec::new();
        for packet in file.packets {
            let Some((codec_id, default_duration_ns)) = tracks.get(&packet.track_number) else {
                continue;
            };
            let entry = (packet, default_duration_ns.unwrap_or(20_000_000));
            match *codec_id {
                "V_VP9" => video_packets.push(entry),
                "A_OPUS" => audio_packets.push(entry),
                _ => {}
            }
        }
        crate::videos_log!(
            "[videos] dispatching {} VP9 and {} Opus packets on separate workers",
            video_packets.len(),
            audio_packets.len()
        );
        let (video_result, audio_result) = std::thread::scope(|scope| {
            let video = scope.spawn(|| process_track_packets(state, "Video", video_packets));
            let audio = scope.spawn(|| process_track_packets(state, "Audio", audio_packets));
            (video.join(), audio.join())
        });
        video_result.map_err(|_| Error::from_hresult(E_FAIL))??;
        audio_result.map_err(|_| Error::from_hresult(E_FAIL))??;
        send_eos(state);
        Ok(())
    }
    #[cfg(not(feature = "webm"))]
    {
        let _ = state;
        Err(Error::from_hresult(E_NOTIMPL))
    }
}

#[cfg(feature = "webm")]
fn process_track_packets(
    state: &FilterState,
    output_id: &'static str,
    packets: Vec<(crate::webm::Packet, u64)>,
) -> windows::core::Result<()> {
    let Some(output) = state.pins.iter().find(|pin| pin.spec.id == output_id) else {
        return Ok(());
    };
    if output.peer.lock().unwrap().is_none() {
        crate::videos_log!("[videos] skipping {} packets: output pin is not connected", output_id);
        return Ok(());
    }
    for (index, (packet, duration_ns)) in packets.into_iter().enumerate() {
        if state.stop_requested.load(Ordering::Acquire) {
            crate::videos_log!("[videos] {} track worker stopped", output_id);
            break;
        }
        if index == 0 {
            crate::videos_log!(
                "[videos] first {} packet: {} bytes, keyframe={}",
                output_id,
                packet.data.len(),
                packet.keyframe
            );
        }
        let start = packet.timestamp_ns / 100;
        let duration = (duration_ns as i64 / 100).max(1);
        let stop = start.saturating_add(duration);
        deliver_sample(output, &packet.data, Some((start, stop)), packet.keyframe)?;
    }
    Ok(())
}

fn start_webm_demux(state: Arc<FilterState>) {
    if state.demux_started.swap(true, Ordering::AcqRel) {
        return;
    }
    state.stop_requested.store(false, Ordering::Release);
    crate::videos_log!("[videos] starting WebM demux worker");
    std::thread::spawn(move || {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| process_webm(&state)));
        match result {
            Ok(Ok(())) => crate::videos_log!("[videos] WebM demux worker finished"),
            Ok(Err(_error)) => crate::videos_log!("[videos] WebM demux worker failed: {_error}"),
            Err(_) => crate::videos_log!("[videos] panic in WebM demux worker"),
        }
    });
}

fn opus_frame_size(packet: &[u8]) -> Option<usize> {
    let toc = *packet.first()?;
    let config = (toc >> 3) as usize;
    let frame_samples: usize = if config >= 16 {
        [120, 240, 480, 960][config & 3]
    } else if config >= 12 {
        if config & 1 == 0 { 480 } else { 960 }
    } else {
        [480, 960, 1920, 2880][config & 3]
    };
    let count = match toc & 3 {
        0 => 1,
        1 | 2 => 2,
        _ => usize::from(*packet.get(1)? & 0x3f),
    };
    let samples = frame_samples.checked_mul(count)?;
    (count != 0 && samples <= 5760).then_some(samples)
}

fn process_codec_input(
    state: &FilterState,
    bytes: &[u8],
    input_time: Option<(i64, i64)>,
) -> windows::core::Result<()> {
    match state.kind {
        FilterKind::OpusDecoder => {
            #[cfg(feature = "opus")]
            {
                let frame_size =
                    opus_frame_size(bytes).ok_or_else(|| Error::from_hresult(E_FAIL))?;
                let mut decoder = state.opus.lock().unwrap();
                if decoder.is_none() {
                    *decoder = Some(
                        crate::opus::OpusDecoder::new(2)
                            .map_err(|_| Error::from_hresult(E_FAIL))?,
                    );
                }
                let decoded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    decoder.as_mut().unwrap().decode(bytes, frame_size)
                }));
                let pcm = match decoded {
                    Ok(Ok(pcm)) => pcm,
                    Ok(Err(_error)) => {
                        crate::videos_log!("[videos] Opus decode error: {_error}");
                        return Err(Error::from_hresult(E_FAIL));
                    }
                    Err(_) => {
                        crate::videos_log!("[videos] panic in Opus decoder");
                        return Err(Error::from_hresult(E_FAIL));
                    }
                };
                let mut pcm16 = Vec::with_capacity(pcm.len() * 2);
                for sample in pcm {
                    let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
                    pcm16.extend_from_slice(&value.to_le_bytes());
                }
                let output = state
                    .pins
                    .iter()
                    .find(|pin| pin.spec.id == "Output")
                    .ok_or_else(|| Error::from_hresult(E_FAIL))?;
                deliver_sample(output, &pcm16, input_time, false)?;
            }
            #[cfg(not(feature = "opus"))]
            return Err(Error::from_hresult(E_NOTIMPL));
        }
        FilterKind::Vp9Decoder => {
            #[cfg(feature = "vp9")]
            {
                #[cfg(feature = "log")]
                let decode_started = std::time::Instant::now();
                let mut decoder = state.vp9.lock().unwrap();
                if decoder.is_none() {
                    *decoder = Some(crate::vp9::Vp9Decoder::new());
                }
                let decoded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    decoder.as_mut().unwrap().decode_packet(bytes)
                }));
                let frames = match decoded {
                    Ok(Ok(frames)) => frames,
                    Ok(Err(_error)) => {
                        crate::videos_log!("[videos] VP9 decode error: {_error}");
                        return Err(Error::from_hresult(E_FAIL));
                    }
                    Err(_) => {
                        crate::videos_log!("[videos] panic in VP9 decoder");
                        return Err(Error::from_hresult(E_FAIL));
                    }
                };
                #[cfg(feature = "log")]
                let decode_elapsed_ms = decode_started.elapsed().as_millis();
                let output = state
                    .pins
                    .iter()
                    .find(|pin| pin.spec.id == "Output")
                    .ok_or_else(|| Error::from_hresult(E_FAIL))?;
                for frame in frames {
                    let number = output.delivered_samples.load(Ordering::Relaxed);
                    if number == 0 || number % 300 == 0 {
                        crate::videos_log!(
                            "[videos] VP9 frame {} decoded in {} ms: {}x{}, {} bytes; negotiated={:?}",
                            number + 1,
                            decode_elapsed_ms,
                            frame.width,
                            frame.height,
                            frame.bgra.len(),
                            *state.video_dimensions.lock().unwrap()
                        );
                    }
                    deliver_sample(output, &frame.bgra, input_time, true)?;
                }
            }
            #[cfg(not(feature = "vp9"))]
            return Err(Error::from_hresult(E_NOTIMPL));
        }
        FilterKind::WebmSplitter => {}
        FilterKind::DisabledAudio => {}
    }
    Ok(())
}

#[implement(IBaseFilter, IMediaFilter)]
struct FilterCom {
    state: Arc<FilterState>,
}

impl IPersist_Impl for FilterCom_Impl {
    fn GetClassID(&self) -> windows::core::Result<GUID> {
        crate::videos_log!("[videos] {} GetClassID", self.state.name);
        Ok(self.state.class_id)
    }
}

impl IMediaFilter_Impl for FilterCom_Impl {
    fn Stop(&self) -> windows::core::Result<()> {
        crate::videos_log!("[videos] {:?} Stop", self.state.kind);
        self.state.stop_requested.store(true, Ordering::Release);
        *self.state.state.lock().unwrap() = State_Stopped;
        Ok(())
    }
    fn Pause(&self) -> windows::core::Result<()> {
        crate::videos_log!("[videos] {:?} Pause", self.state.kind);
        *self.state.state.lock().unwrap() = State_Paused;
        // DirectShow waits for a preroll sample while transitioning the graph
        // from Stopped to Paused. A pull source must therefore start producing
        // before Run, otherwise IMediaControl::Run remains pending forever.
        if matches!(self.state.kind, FilterKind::WebmSplitter)
            && self.state.pull_source.load(Ordering::Acquire)
        {
            crate::videos_log!("[videos] starting pull-source WebM demux from Pause");
            start_webm_demux(self.state.clone());
        }
        Ok(())
    }
    fn Run(&self, _tstart: i64) -> windows::core::Result<()> {
        crate::videos_log!("[videos] {:?} Run at {_tstart}", self.state.kind);
        *self.state.state.lock().unwrap() = State_Running;
        if matches!(self.state.kind, FilterKind::WebmSplitter)
            && self.state.pull_source.load(Ordering::Acquire)
        {
            crate::videos_log!("[videos] starting pull-source WebM demux from Run");
            start_webm_demux(self.state.clone());
        }
        Ok(())
    }
    fn GetState(&self, _timeout: u32) -> windows::core::Result<FILTER_STATE> {
        Ok(*self.state.state.lock().unwrap())
    }
    fn SetSyncSource(&self, _clock: Ref<IReferenceClock>) -> windows::core::Result<()> {
        Ok(())
    }
    fn GetSyncSource(&self) -> windows::core::Result<IReferenceClock> {
        Err(Error::from_hresult(VFW_E_NOT_CONNECTED))
    }
}

impl IBaseFilter_Impl for FilterCom_Impl {
    fn EnumPins(&self) -> windows::core::Result<IEnumPins> {
        crate::videos_log!("[videos] {} EnumPins", self.state.name);
        let pins = stable_pin_interfaces(self);
        let enumerator = ComObject::new(PinEnumerator {
            pins,
            index: Mutex::new(0),
        });
        Ok(enumerator.into_interface())
    }

    fn FindPin(&self, id: &PCWSTR) -> windows::core::Result<IPin> {
        let id = unsafe { id.to_string() }.map_err(|_| Error::from_hresult(E_INVALIDARG))?;
        let Some(index) = self.state.pins.iter().position(|pin| pin.spec.id == id) else {
            return Err(Error::from_hresult(VFW_E_NOT_FOUND));
        };
        crate::videos_log!("[videos] {} FindPin({})", self.state.name, id);
        stable_pin_interfaces(self)
            .get(index)
            .cloned()
            .ok_or_else(|| Error::from_hresult(E_FAIL))
    }

    fn QueryFilterInfo(&self, info: *mut FILTER_INFO) -> windows::core::Result<()> {
        crate::videos_log!("[videos] {} QueryFilterInfo", self.state.name);
        if info.is_null() {
            return Err(Error::from_hresult(E_POINTER));
        }
        unsafe {
            let target = &mut *info;
            target.achName = [0; 128];
            for (dst, src) in target
                .achName
                .iter_mut()
                .zip(self.state.name.encode_utf16())
            {
                *dst = src;
            }
            target.pGraph = ManuallyDrop::new(self.state.graph.lock().unwrap().clone());
        }
        Ok(())
    }

    fn JoinFilterGraph(
        &self,
        graph: Ref<IFilterGraph>,
        _name: &PCWSTR,
    ) -> windows::core::Result<()> {
        crate::videos_log!("[videos] {} JoinFilterGraph", self.state.name);
        *self.state.graph.lock().unwrap() = graph.as_ref().cloned();
        Ok(())
    }

    fn QueryVendorInfo(&self) -> windows::core::Result<PWSTR> {
        Err(Error::from_hresult(E_NOTIMPL))
    }
}

fn stable_pin_interfaces(filter: &FilterCom_Impl) -> Vec<IPin> {
    let mut pins = filter.state.pin_interfaces.lock().unwrap();
    if pins.is_empty() {
        // Pin objects keep only this non-owning pointer to their parent. Their
        // weak FilterState relationship makes it valid while calls are active
        // and avoids a Filter -> Pin -> Filter COM reference cycle.
        let owner = filter.to_object().to_interface::<IBaseFilter>();
        let owner_ptr = owner.as_raw();
        drop(owner);
        for state in &filter.state.pins {
            let pin: IPin = ComObject::new(PinCom {
                owner: owner_ptr,
                state: state.clone(),
            })
            .into_interface();
            unsafe { install_connected_to_thunk(&pin) };
            pins.push(pin);
        }
    }
    pins.clone()
}

static PIN_VTABLE_WITH_CONNECTED_TO: OnceLock<usize> = OnceLock::new();

unsafe fn install_connected_to_thunk(pin: &IPin) {
    let interface = pin.as_raw();
    let original = unsafe { *(interface.cast::<*const IPin_Vtbl>()) };
    let replacement = *PIN_VTABLE_WITH_CONNECTED_TO.get_or_init(|| unsafe {
        let mut vtable = original.read();
        vtable.ConnectedTo = pin_connected_to;
        Box::into_raw(Box::new(vtable)) as usize
    }) as *const IPin_Vtbl;
    unsafe { interface.cast::<*const IPin_Vtbl>().write(replacement) };
}

unsafe extern "system" fn pin_connected_to(this: *mut c_void, output: *mut *mut c_void) -> HRESULT {
    crate::videos_log!("[videos] native ConnectedTo thunk this={this:p}");
    if output.is_null() {
        return E_POINTER;
    }
    unsafe { output.write(ptr::null_mut()) };

    let borrowed = ManuallyDrop::new(unsafe { IPin::from_raw(this) });
    let object = match ComObject::<PinCom>::cast_from(&*borrowed) {
        Ok(object) => object,
        Err(error) => return error.code(),
    };
    let peer = object.state.peer.lock().unwrap().clone();
    match peer {
        Some(peer) => {
            unsafe { output.write(peer.into_raw()) };
            crate::videos_log!("[videos] native ConnectedTo returned connected peer");
            S_OK
        }
        None => {
            crate::videos_log!(
                "[videos] native ConnectedTo wrote null; returning VFW_E_NOT_CONNECTED"
            );
            VFW_E_NOT_CONNECTED
        }
    }
}

#[implement(IEnumPins)]
struct PinEnumerator {
    pins: Vec<IPin>,
    index: Mutex<usize>,
}

impl IEnumPins_Impl for PinEnumerator_Impl {
    fn Next(&self, count: u32, out: *mut Option<IPin>, fetched: *mut u32) -> HRESULT {
        crate::videos_log!(
            "[videos] IEnumPins::Next count={count}, out={out:p}, fetched={fetched:p}"
        );
        if out.is_null() || (count != 1 && fetched.is_null()) {
            crate::videos_log!("[videos] IEnumPins::Next invalid output pointers");
            return E_POINTER;
        }
        // Clear the full caller-provided output range. COM clients should use
        // `fetched`, but leaving unused slots uninitialized can crash clients
        // that inspect or release the whole requested array.
        for slot in 0..count as usize {
            unsafe {
                out.add(slot).write(None);
            }
        }
        let mut index = self.index.lock().unwrap();
        let mut copied = 0u32;
        while copied < count && *index < self.pins.len() {
            crate::videos_log!("[videos] IEnumPins::Next returning pin at slot {}", copied);
            unsafe {
                out.add(copied as usize)
                    .write(Some(self.pins[*index].clone()));
            }
            crate::videos_log!("[videos] IEnumPins::Next pin written");
            *index += 1;
            copied += 1;
        }
        if !fetched.is_null() {
            unsafe {
                fetched.write(copied);
            }
        }
        let result = if copied == count { S_OK } else { S_FALSE };
        crate::videos_log!(
            "[videos] IEnumPins::Next returning {:?}, fetched={copied}",
            result
        );
        result
    }
    fn Skip(&self, count: u32) -> windows::core::Result<()> {
        let mut index = self.index.lock().unwrap();
        *index = (*index + count as usize).min(self.pins.len());
        if *index == self.pins.len() {
            Err(Error::from_hresult(S_FALSE))
        } else {
            Ok(())
        }
    }
    fn Reset(&self) -> windows::core::Result<()> {
        *self.index.lock().unwrap() = 0;
        Ok(())
    }
    fn Clone(&self) -> windows::core::Result<IEnumPins> {
        let enumerator = ComObject::new(PinEnumerator {
            pins: self.pins.clone(),
            index: Mutex::new(*self.index.lock().unwrap()),
        });
        Ok(enumerator.into_interface())
    }
}

#[implement(IPin, IMemInputPin)]
struct PinCom {
    owner: *mut c_void,
    state: Arc<PinState>,
}

impl IPin_Impl for PinCom_Impl {
    fn Connect(
        &self,
        receive: Ref<IPin>,
        media_type: *const AM_MEDIA_TYPE,
    ) -> windows::core::Result<()> {
        crate::videos_log!("[videos] Connect output pin {}", self.state.spec.id);
        if self.state.spec.direction != PINDIR_OUTPUT {
            return Err(Error::from_hresult(VFW_E_INVALID_DIRECTION));
        }
        let receive = receive.ok()?.clone();
        if media_type.is_null() {
            let available_types = resolved_media_types(&self.state);
            let spec = available_types
                .first()
                .ok_or_else(|| Error::from_hresult(VFW_E_NO_ACCEPTABLE_TYPES))?;
            let mut mt = make_media_type(*spec)?;
            let result =
                unsafe { receive.ReceiveConnection(&self.to_object().to_interface::<IPin>(), &mt) };
            free_media_type_format(&mut mt);
            result?;
            *self.state.connection_type.lock().unwrap() = Some(*spec);
        } else {
            unsafe {
                receive
                    .ReceiveConnection(&self.to_object().to_interface::<IPin>(), &*media_type)?;
            }
            *self.state.connection_type.lock().unwrap() = Some(MediaSpec {
                major: unsafe { (*media_type).majortype },
                subtype: unsafe { (*media_type).subtype },
                format: media_format_from_type(unsafe { &*media_type }).unwrap_or(FormatKind::None),
            });
        }
        connect_allocator(&self.state, &receive)?;
        *self.state.peer.lock().unwrap() = Some(receive);
        Ok(())
    }

    fn ReceiveConnection(
        &self,
        connector: Ref<IPin>,
        media_type: *const AM_MEDIA_TYPE,
    ) -> windows::core::Result<()> {
        crate::videos_log!("[videos] ReceiveConnection on pin {}", self.state.spec.id);
        if self.state.spec.direction != PINDIR_INPUT {
            return Err(Error::from_hresult(VFW_E_INVALID_DIRECTION));
        }
        if media_type.is_null() {
            return Err(Error::from_hresult(E_POINTER));
        }
        let mt = unsafe { &*media_type };
        crate::videos_log!(
            "[videos] ReceiveConnection media type {}: mt={:p}, major={:?}, subtype={:?}, format={:?}, cbFormat={}, pbFormat={:p}",
            self.state.spec.id,
            media_type,
            mt.majortype,
            mt.subtype,
            mt.formattype,
            mt.cbFormat,
            mt.pbFormat
        );
        if self.QueryAccept(media_type) != S_OK {
            return Err(Error::from_hresult(VFW_E_TYPE_NOT_ACCEPTED));
        }
        let connector = connector.ok()?.clone();
        if matches!(
            self.state.owner.upgrade().map(|owner| owner.kind),
            Some(FilterKind::WebmSplitter)
        ) && self.state.spec.id == "Input"
        {
            // krkrz's CIStreamReader is an asynchronous pull source. Querying
            // IAsyncReader is required for its output pin to finish connecting.
            let reader: IAsyncReader = connector.cast()?;
            #[cfg(feature = "webm")]
            {
                if let Some(owner) = self.state.owner.upgrade() {
                    owner.pull_source.store(true, Ordering::Release);
                }
                let (mut total, mut available) = (0i64, 0i64);
                unsafe {
                    reader.Length(&mut total, &mut available)?;
                }
                if total <= 0 || total > 512 * 1024 * 1024 {
                    crate::videos_log!("[videos] invalid async source length: {total}");
                    return Err(Error::from_hresult(E_INVALIDARG));
                }
                let mut bytes = vec![0u8; total as usize];
                for (index, chunk) in bytes.chunks_mut(1024 * 1024).enumerate() {
                    unsafe {
                        reader.SyncRead((index * 1024 * 1024) as i64, chunk)?;
                    }
                }
                crate::videos_log!("[videos] read {total} bytes through IAsyncReader");
                let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    crate::webm::WebmFile::parse(&bytes)
                }));
                let file = match parsed {
                    Ok(Ok(file)) => file,
                    Ok(Err(_error)) => {
                        crate::videos_log!("[videos] WebM header parse error: {_error:?}");
                        return Err(Error::from_hresult(E_FAIL));
                    }
                    Err(_) => {
                        crate::videos_log!("[videos] panic while reading WebM header");
                        return Err(Error::from_hresult(E_FAIL));
                    }
                };
                if let Some(track) = file.tracks.iter().find(|track| track.codec_id == "V_VP9") {
                    let (Some(width), Some(height)) = (track.width, track.height) else {
                        crate::videos_log!("[videos] VP9 track has no pixel dimensions");
                        return Err(Error::from_hresult(E_INVALIDARG));
                    };
                    let (width, height) = (
                        u32::try_from(width).map_err(|_| Error::from_hresult(E_INVALIDARG))?,
                        u32::try_from(height).map_err(|_| Error::from_hresult(E_INVALIDARG))?,
                    );
                    if width == 0 || height == 0 || width > 16_384 || height > 16_384 {
                        return Err(Error::from_hresult(E_INVALIDARG));
                    }
                    let owner = self
                        .state
                        .owner
                        .upgrade()
                        .ok_or_else(|| Error::from_hresult(E_FAIL))?;
                    *owner.video_dimensions.lock().unwrap() = Some((width, height));
                    crate::videos_log!("[videos] WebM VP9 dimensions: {width}x{height}");
                }
                *self.state.received.lock().unwrap() = bytes;
                let _ = available;
            }
            #[cfg(not(feature = "webm"))]
            {
                let _ = reader;
                return Err(Error::from_hresult(E_NOTIMPL));
            }
        }
        *self.state.peer.lock().unwrap() = Some(connector);
        *self.state.connection_type.lock().unwrap() = Some(MediaSpec {
            major: mt.majortype,
            subtype: mt.subtype,
            format: media_format_from_type(mt).unwrap_or(FormatKind::None),
        });
        if self.state.spec.id == "Input"
            && matches!(
                self.state.owner.upgrade().map(|owner| owner.kind),
                Some(FilterKind::Vp9Decoder)
            )
        {
            if let Some(FormatKind::Vp9 { width, height }) = media_format_from_type(mt) {
                let owner = self
                    .state
                    .owner
                    .upgrade()
                    .ok_or_else(|| Error::from_hresult(E_FAIL))?;
                *owner.video_dimensions.lock().unwrap() = Some((width, height));
                crate::videos_log!("[videos] VP9 decoder negotiated {width}x{height}");
            } else {
                crate::videos_log!("[videos] VP9 media type format block was not recognized");
            }
        }
        Ok(())
    }

    fn Disconnect(&self) -> windows::core::Result<()> {
        *self.state.peer.lock().unwrap() = None;
        *self.state.connection_type.lock().unwrap() = None;
        Ok(())
    }
    fn ConnectedTo(&self) -> windows::core::Result<IPin> {
        crate::videos_log!("[videos] ConnectedTo {}", self.state.spec.id);
        let peer = self.state.peer.lock().unwrap().clone();
        crate::videos_log!(
            "[videos] ConnectedTo {} lock acquired; connected={}",
            self.state.spec.id,
            peer.is_some()
        );
        match peer {
            Some(peer) => Ok(peer),
            None => {
                crate::videos_log!(
                    "[videos] ConnectedTo {} returning VFW_E_NOT_CONNECTED",
                    self.state.spec.id
                );
                Err(Error::from_hresult(VFW_E_NOT_CONNECTED))
            }
        }
    }
    fn ConnectionMediaType(&self, out: *mut AM_MEDIA_TYPE) -> windows::core::Result<()> {
        if out.is_null() {
            return Err(Error::from_hresult(E_POINTER));
        }
        let spec = self
            .state
            .connection_type
            .lock()
            .unwrap()
            .ok_or_else(|| Error::from_hresult(VFW_E_NOT_CONNECTED))?;
        unsafe {
            out.write(make_media_type(spec)?);
        }
        Ok(())
    }
    fn QueryPinInfo(&self, info: *mut PIN_INFO) -> windows::core::Result<()> {
        crate::videos_log!("[videos] QueryPinInfo {}", self.state.spec.id);
        if info.is_null() {
            return Err(Error::from_hresult(E_POINTER));
        }
        if self.state.owner.upgrade().is_none() {
            return Err(Error::from_hresult(E_FAIL));
        }
        let mut value = PIN_INFO::default();
        let borrowed_owner = ManuallyDrop::new(unsafe { IBaseFilter::from_raw(self.owner) });
        value.pFilter = ManuallyDrop::new(Some((*borrowed_owner).clone()));
        value.dir = self.state.spec.direction;
        for (dst, src) in value
            .achName
            .iter_mut()
            .zip(self.state.spec.name.encode_utf16())
        {
            *dst = src;
        }
        unsafe {
            info.write(value);
        }
        Ok(())
    }
    fn QueryDirection(&self) -> windows::core::Result<PIN_DIRECTION> {
        crate::videos_log!("[videos] QueryDirection {}", self.state.spec.id);
        Ok(self.state.spec.direction)
    }
    fn QueryId(&self) -> windows::core::Result<PWSTR> {
        let wide: Vec<u16> = self.state.spec.id.encode_utf16().chain(Some(0)).collect();
        let size = wide.len() * std::mem::size_of::<u16>();
        let ptr = unsafe { CoTaskMemAlloc(size).cast::<u16>() };
        if ptr.is_null() {
            return Err(Error::from_hresult(E_OUTOFMEMORY));
        }
        unsafe {
            ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        }
        Ok(PWSTR(ptr))
    }
    fn QueryAccept(&self, mt: *const AM_MEDIA_TYPE) -> HRESULT {
        crate::videos_log!("[videos] QueryAccept {}", self.state.spec.id);
        if mt.is_null() {
            return E_POINTER;
        }
        let mt = unsafe { &*mt };
        if self.state.spec.media.iter().any(|candidate| {
            candidate.major == mt.majortype
                && (candidate.subtype == GUID::zeroed() || candidate.subtype == mt.subtype)
        }) {
            S_OK
        } else {
            crate::videos_log!(
                "[videos] rejected media type on pin {}: {:?}/{:?}",
                self.state.spec.id,
                mt.majortype,
                mt.subtype
            );
            VFW_E_TYPE_NOT_ACCEPTED
        }
    }
    fn EnumMediaTypes(&self) -> windows::core::Result<IEnumMediaTypes> {
        crate::videos_log!("[videos] EnumMediaTypes {}", self.state.spec.id);
        let enum_types = ComObject::new(MediaTypeEnumerator {
            types: resolved_media_types(&self.state),
            index: Mutex::new(0),
        });
        Ok(enum_types.into_interface())
    }
    fn QueryInternalConnections(
        &self,
        _pins: windows::core::OutRef<IPin>,
        _count: *mut u32,
    ) -> windows::core::Result<()> {
        Err(Error::from_hresult(E_NOTIMPL))
    }
    fn EndOfStream(&self) -> windows::core::Result<()> {
        crate::videos_log!("[videos] EndOfStream on pin {}", self.state.spec.id);
        let Some(owner) = self.state.owner.upgrade() else {
            return Ok(());
        };
        match owner.kind {
            FilterKind::WebmSplitter if self.state.spec.direction == PINDIR_INPUT => {
                start_webm_demux(owner)
            }
            FilterKind::Vp9Decoder | FilterKind::OpusDecoder
                if self.state.spec.direction == PINDIR_INPUT =>
            {
                send_eos(&owner)
            }
            _ => {}
        }
        Ok(())
    }
    fn BeginFlush(&self) -> windows::core::Result<()> {
        self.state.received.lock().unwrap().clear();
        Ok(())
    }
    fn EndFlush(&self) -> windows::core::Result<()> {
        Ok(())
    }
    fn NewSegment(&self, _start: i64, _stop: i64, _rate: f64) -> windows::core::Result<()> {
        Ok(())
    }
}

impl IMemInputPin_Impl for PinCom_Impl {
    fn GetAllocator(&self) -> windows::core::Result<IMemAllocator> {
        input_allocator(&self.state)
    }
    fn NotifyAllocator(
        &self,
        allocator: Ref<IMemAllocator>,
        _read_only: BOOL,
    ) -> windows::core::Result<()> {
        *self.state.allocator.lock().unwrap() = allocator.as_ref().cloned();
        Ok(())
    }
    fn GetAllocatorRequirements(&self) -> windows::core::Result<ALLOCATOR_PROPERTIES> {
        Ok(ALLOCATOR_PROPERTIES::default())
    }
    fn Receive(&self, sample: Ref<IMediaSample>) -> windows::core::Result<()> {
        if self.state.spec.direction != PINDIR_INPUT {
            return Err(Error::from_hresult(VFW_E_INVALID_DIRECTION));
        }
        let sample = sample.ok()?;
        let data = unsafe { sample.GetPointer()? };
        let len = unsafe { sample.GetActualDataLength() };
        let number = self.state.received_samples.fetch_add(1, Ordering::Relaxed) + 1;
        if number == 1 || number % 300 == 0 {
            crate::videos_log!(
                "[videos] received {} sample {} ({} bytes)",
                self.state.spec.id,
                number,
                len
            );
        }
        if len < 0 || (len != 0 && data.is_null()) {
            return Err(Error::from_hresult(E_FAIL));
        }
        if len != 0 {
            let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len as usize) };
            let owner = self
                .state
                .owner
                .upgrade()
                .ok_or_else(|| Error::from_hresult(E_FAIL))?;
            match owner.kind {
                FilterKind::WebmSplitter => {
                    self.state.received.lock().unwrap().extend_from_slice(bytes)
                }
                FilterKind::Vp9Decoder | FilterKind::OpusDecoder => {
                    let mut start = 0i64;
                    let mut stop = 0i64;
                    let time = unsafe { sample.GetTime(&mut start, &mut stop) }
                        .ok()
                        .map(|_| (start, stop));
                    process_codec_input(&owner, bytes, time)?;
                }
                FilterKind::DisabledAudio => return Err(Error::from_hresult(E_NOTIMPL)),
            }
        }
        Ok(())
    }
    fn ReceiveMultiple(
        &self,
        samples: *const Option<IMediaSample>,
        count: i32,
    ) -> windows::core::Result<i32> {
        if samples.is_null() || count < 0 {
            return Err(Error::from_hresult(E_INVALIDARG));
        }
        for i in 0..count as usize {
            let sample = unsafe { (*samples.add(i)).as_ref() }
                .ok_or_else(|| Error::from_hresult(E_POINTER))?;
            self.Receive(Ref::from(sample))?;
        }
        Ok(count)
    }
    fn ReceiveCanBlock(&self) -> windows::core::Result<()> {
        Ok(())
    }
}

#[implement(IEnumMediaTypes)]
struct MediaTypeEnumerator {
    types: Vec<MediaSpec>,
    index: Mutex<usize>,
}

impl IEnumMediaTypes_Impl for MediaTypeEnumerator_Impl {
    fn Next(&self, count: u32, output: *mut *mut AM_MEDIA_TYPE, fetched: *mut u32) -> HRESULT {
        if output.is_null() || (count != 1 && fetched.is_null()) {
            return E_POINTER;
        }
        let mut index = self.index.lock().unwrap();
        let mut copied = 0;
        while copied < count && *index < self.types.len() {
            let media_type = match make_media_type(self.types[*index]) {
                Ok(media_type) => media_type,
                Err(error) => return error.code(),
            };
            if self.types[*index].major == MEDIATYPE_Video {
                let _dimensions = media_format_from_type(&media_type);
                crate::videos_log!(
                    "[videos] IEnumMediaTypes::Next video format index={}, mt={:p}, cbFormat={}, pbFormat={:p}, dimensions={:?}",
                    *index,
                    &media_type,
                    media_type.cbFormat,
                    media_type.pbFormat,
                    _dimensions.map(|format| match format {
                        FormatKind::Vp9 { width, height } | FormatKind::Rgb32 { width, height } => (width, height),
                        _ => (0, 0),
                    })
                );
            }
            let ptr = unsafe {
                CoTaskMemAlloc(std::mem::size_of::<AM_MEDIA_TYPE>()).cast::<AM_MEDIA_TYPE>()
            };
            if ptr.is_null() {
                let mut media_type = media_type;
                free_media_type_format(&mut media_type);
                return E_OUTOFMEMORY;
            }
            unsafe {
                ptr.write(media_type);
                output.add(copied as usize).write(ptr);
            }
            *index += 1;
            copied += 1;
        }
        if !fetched.is_null() {
            unsafe {
                fetched.write(copied);
            }
        }
        if copied == count { S_OK } else { S_FALSE }
    }
    fn Skip(&self, count: u32) -> windows::core::Result<()> {
        let mut index = self.index.lock().unwrap();
        *index = (*index + count as usize).min(self.types.len());
        if *index == self.types.len() {
            Err(Error::from_hresult(S_FALSE))
        } else {
            Ok(())
        }
    }
    fn Reset(&self) -> windows::core::Result<()> {
        *self.index.lock().unwrap() = 0;
        Ok(())
    }
    fn Clone(&self) -> windows::core::Result<IEnumMediaTypes> {
        let value = ComObject::new(MediaTypeEnumerator {
            types: self.types.clone(),
            index: Mutex::new(*self.index.lock().unwrap()),
        });
        Ok(value.into_interface())
    }
}

fn make_media_type(spec: MediaSpec) -> windows::core::Result<AM_MEDIA_TYPE> {
    let (formattype, format, sample_size, fixed, temporal) = match spec.format {
        FormatKind::None => (GUID::zeroed(), None, 0, false, true),
        FormatKind::Vp9 { width, height } | FormatKind::Rgb32 { width, height } => {
            let rgb32 = matches!(spec.format, FormatKind::Rgb32 { .. });
            let image_size = width
                .checked_mul(height)
                .and_then(|pixels| pixels.checked_mul(if rgb32 { 4 } else { 1 }))
                .ok_or_else(|| Error::from_hresult(E_INVALIDARG))?;
            let mut info = VideoInfoHeader::default();
            info.rcSource = RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            info.rcTarget = info.rcSource;
            info.AvgTimePerFrame = 333_333;
            info.bmiHeader = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: height as i32,
                biPlanes: 1,
                biBitCount: if rgb32 { 32 } else { 0 },
                biCompression: if rgb32 { 0 } else { fourcc(b"VP90").data1 },
                biSizeImage: image_size,
                ..BITMAPINFOHEADER::default()
            };
            let format = unsafe {
                std::slice::from_raw_parts(
                    (&info as *const VideoInfoHeader).cast::<u8>(),
                    std::mem::size_of::<VideoInfoHeader>(),
                )
                .to_vec()
            };
            (
                FORMAT_VideoInfo,
                Some(format),
                if rgb32 { image_size } else { 0 },
                rgb32,
                !rgb32,
            )
        }
        FormatKind::Pcm16Stereo48k => {
            let wave = WaveFormatEx {
                format_tag: 1,
                channels: 2,
                samples_per_sec: 48_000,
                avg_bytes_per_sec: 192_000,
                block_align: 4,
                bits_per_sample: 16,
                extra_size: 0,
            };
            let format = unsafe {
                std::slice::from_raw_parts(
                    (&wave as *const WaveFormatEx).cast::<u8>(),
                    std::mem::size_of::<WaveFormatEx>(),
                )
                .to_vec()
            };
            (
                FORMAT_WaveFormatEx,
                Some(format),
                4,
                true,
                false,
            )
        }
    };
    let (format_data, format_size) = if let Some(bytes) = format {
        let size = bytes.len();
        let destination = unsafe { CoTaskMemAlloc(size).cast::<u8>() };
        if destination.is_null() {
            return Err(Error::from_hresult(E_OUTOFMEMORY));
        }
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), destination, size);
        }
        (destination, size as u32)
    } else {
        (ptr::null_mut(), 0)
    };
    Ok(AM_MEDIA_TYPE {
        majortype: spec.major,
        subtype: spec.subtype,
        bFixedSizeSamples: BOOL(i32::from(fixed)),
        bTemporalCompression: BOOL(i32::from(temporal)),
        lSampleSize: sample_size as u32,
        formattype,
        pUnk: ManuallyDrop::new(None::<IUnknown>),
        cbFormat: format_size,
        pbFormat: format_data,
    })
}

fn media_format_from_type(mt: &AM_MEDIA_TYPE) -> Option<FormatKind> {
    if mt.formattype != FORMAT_VideoInfo
        || mt.pbFormat.is_null()
        || (mt.cbFormat as usize) < std::mem::size_of::<VideoInfoHeader>()
    {
        return None;
    }
    let info = unsafe { &*mt.pbFormat.cast::<VideoInfoHeader>() };
    let width = info.bmiHeader.biWidth.unsigned_abs();
    let height = info.bmiHeader.biHeight.unsigned_abs();
    if width == 0 || height == 0 || width > 16_384 || height > 16_384 {
        return None;
    }
    if mt.subtype == MEDIASUBTYPE_RGB32 {
        Some(FormatKind::Rgb32 { width, height })
    } else if mt.subtype == fourcc(b"VP90") {
        Some(FormatKind::Vp9 { width, height })
    } else {
        None
    }
}

fn free_media_type_format(media_type: &mut AM_MEDIA_TYPE) {
    if !media_type.pbFormat.is_null() {
        unsafe {
            CoTaskMemFree(Some(media_type.pbFormat.cast()));
        }
        media_type.pbFormat = ptr::null_mut();
        media_type.cbFormat = 0;
    }
}

pub(crate) fn create_filter(kind: FilterKind) -> *mut c_void {
    crate::videos_log!("[videos] creating {:?} DirectShow filter", kind);
    let state = filter_state(kind);
    crate::videos_log!("[videos] constructing {:?} COM object", kind);
    let object = ComObject::new(FilterCom { state });
    crate::videos_log!("[videos] {:?} COM object constructed", kind);
    let interface = object.into_interface::<IBaseFilter>();
    crate::videos_log!("[videos] {:?} IBaseFilter interface created", kind);
    let raw = interface.into_raw();
    crate::videos_log!("[videos] {:?} factory returning {:p}", kind, raw);
    raw
}

pub(crate) unsafe extern "C-unwind" fn splitter_factory(_format_data: *mut c_void) -> *mut c_void {
    create_filter(FilterKind::WebmSplitter)
}
pub(crate) unsafe extern "C-unwind" fn vp9_factory(_format_data: *mut c_void) -> *mut c_void {
    create_filter(FilterKind::Vp9Decoder)
}
pub(crate) unsafe extern "C-unwind" fn opus_factory(_format_data: *mut c_void) -> *mut c_void {
    #[cfg(feature = "opus")]
    return create_filter(FilterKind::OpusDecoder);
    #[cfg(not(feature = "opus"))]
    return create_filter(FilterKind::DisabledAudio);
}
