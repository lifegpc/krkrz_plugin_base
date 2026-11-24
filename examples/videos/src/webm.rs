//! Minimal WebM/Matroska demuxing for VP9 and Opus tracks.

use std::collections::HashMap;

const EBML: u64 = 0x1A45DFA3;
const SEGMENT: u64 = 0x18538067;
const INFO: u64 = 0x1549A966;
const TRACKS: u64 = 0x1654AE6B;
const TRACK_ENTRY: u64 = 0xAE;
const CLUSTER: u64 = 0x1F43B675;
const SIMPLE_BLOCK: u64 = 0xA3;
const BLOCK_GROUP: u64 = 0xA0;
const BLOCK: u64 = 0xA1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackKind {
    Video,
    Audio,
    Other,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub number: u64,
    pub kind: TrackKind,
    pub codec_id: String,
    pub codec_private: Vec<u8>,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub channels: Option<u64>,
    pub sample_rate: Option<f64>,
    pub default_duration_ns: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub track_number: u64,
    pub timestamp_ns: i64,
    pub keyframe: bool,
    pub invisible: bool,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    InvalidVint,
    Truncated,
    InvalidElement,
    InvalidBlock,
    UnsupportedLacing,
    MissingSegment,
}

#[derive(Debug, Default)]
pub struct WebmFile {
    pub tracks: Vec<Track>,
    pub timecode_scale_ns: u64,
    pub duration_ticks: Option<f64>,
    pub packets: Vec<Packet>,
}

#[derive(Clone, Copy)]
struct Element<'a> {
    id: u64,
    data: &'a [u8],
}

impl WebmFile {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        let mut file = Self {
            timecode_scale_ns: 1_000_000,
            ..Self::default()
        };
        let mut saw_segment = false;
        for element in elements(bytes)? {
            let element = element?;
            if element.id == EBML {
                continue;
            }
            if element.id != SEGMENT {
                continue;
            }
            saw_segment = true;
            for child in elements(element.data)? {
                let child = child?;
                match child.id {
                    INFO => file.parse_info(child.data)?,
                    TRACKS => file.parse_tracks(child.data)?,
                    CLUSTER => file.parse_cluster(child.data)?,
                    _ => {}
                }
            }
        }
        if !saw_segment {
            return Err(Error::MissingSegment);
        }
        Ok(file)
    }

    fn parse_info(&mut self, data: &[u8]) -> Result<(), Error> {
        for item in elements(data)? {
            let item = item?;
            match item.id {
                0x2AD7B1 => self.timecode_scale_ns = uint(item.data)?,
                0x4489 => self.duration_ticks = Some(float(item.data)?),
                _ => {}
            }
        }
        Ok(())
    }

    fn parse_tracks(&mut self, data: &[u8]) -> Result<(), Error> {
        for item in elements(data)? {
            let item = item?;
            if item.id != TRACK_ENTRY {
                continue;
            }
            let mut track = Track {
                number: 0,
                kind: TrackKind::Other,
                codec_id: String::new(),
                codec_private: Vec::new(),
                width: None,
                height: None,
                channels: None,
                sample_rate: None,
                default_duration_ns: None,
            };
            for field in elements(item.data)? {
                let field = field?;
                match field.id {
                    0xD7 => track.number = uint(field.data)?,
                    0x83 => {
                        track.kind = match uint(field.data)? {
                            1 => TrackKind::Video,
                            2 => TrackKind::Audio,
                            _ => TrackKind::Other,
                        }
                    }
                    0x86 => track.codec_id = text(field.data)?,
                    0x63A2 => track.codec_private = field.data.to_vec(),
                    0x23E383 => track.default_duration_ns = Some(uint(field.data)?),
                    0xE0 => {
                        for v in elements(field.data)? {
                            let v = v?;
                            match v.id {
                                0xB0 => track.width = Some(uint(v.data)?),
                                0xBA => track.height = Some(uint(v.data)?),
                                _ => {}
                            }
                        }
                    }
                    0xE1 => {
                        for v in elements(field.data)? {
                            let v = v?;
                            match v.id {
                                0x9F => track.channels = Some(uint(v.data)?),
                                0xB5 => track.sample_rate = Some(float(v.data)?),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            if track.number == 0 || track.codec_id.is_empty() {
                return Err(Error::InvalidElement);
            }
            self.tracks.push(track);
        }
        Ok(())
    }

    fn parse_cluster(&mut self, data: &[u8]) -> Result<(), Error> {
        let mut cluster_time = 0i64;
        let mut pending_blocks = Vec::new();
        for item in elements(data)? {
            let item = item?;
            match item.id {
                0xE7 => {
                    cluster_time =
                        i64::try_from(uint(item.data)?).map_err(|_| Error::InvalidElement)?
                }
                SIMPLE_BLOCK => pending_blocks.extend(parse_block(item.data, None)?),
                BLOCK_GROUP => {
                    let mut block = None;
                    let mut has_reference = false;
                    for part in elements(item.data)? {
                        let part = part?;
                        match part.id {
                            BLOCK => block = Some(part.data),
                            0xFB => has_reference = true,
                            _ => {}
                        }
                    }
                    if let Some(block) = block {
                        pending_blocks.extend(parse_block(block, Some(!has_reference))?);
                    }
                }
                _ => {}
            }
        }
        for (track, relative, key, invisible, data) in pending_blocks {
            let ticks = cluster_time
                .checked_add(relative as i64)
                .ok_or(Error::InvalidBlock)?;
            let timestamp_ns = ticks
                .checked_mul(self.timecode_scale_ns as i64)
                .ok_or(Error::InvalidBlock)?;
            self.packets.push(Packet {
                track_number: track,
                timestamp_ns,
                keyframe: key,
                invisible,
                data,
            });
        }
        Ok(())
    }
}

fn parse_block(
    data: &[u8],
    keyframe: Option<bool>,
) -> Result<Vec<(u64, i16, bool, bool, Vec<u8>)>, Error> {
    // The block track number is an EBML VINT value, not an element ID.
    let (track, n) = vint(data, 0, false)?;
    if data.len() < n + 3 {
        return Err(Error::Truncated);
    }
    let relative = i16::from_be_bytes([data[n], data[n + 1]]);
    let flags = data[n + 2];
    let lacing = (flags >> 1) & 3;
    let mut cursor = n + 3;
    let mut sizes = Vec::new();
    match lacing {
        0 => sizes.push(data.len() - cursor),
        1 | 2 | 3 => {
            let count = *data.get(cursor).ok_or(Error::Truncated)? as usize + 1;
            cursor += 1;
            if lacing == 2 {
                let each = (data.len() - cursor) / count;
                sizes.resize(count, each);
            } else if lacing == 1 {
                for _ in 0..count - 1 {
                    let mut size = 0usize;
                    loop {
                        let b = *data.get(cursor).ok_or(Error::Truncated)? as usize;
                        cursor += 1;
                        size = size.checked_add(b).ok_or(Error::InvalidBlock)?;
                        if b != 255 {
                            break;
                        }
                    }
                    sizes.push(size);
                }
                sizes.push(
                    data.len()
                        .checked_sub(cursor + sizes.iter().sum::<usize>())
                        .ok_or(Error::InvalidBlock)?,
                );
            } else {
                let (first, used) = vint(data, cursor, false)?;
                cursor += used;
                sizes.push(first as usize);
                for _ in 1..count - 1 {
                    let (encoded, used) = vint(data, cursor, false)?;
                    cursor += used;
                    let bias = (1i64 << (7 * used - 1)) - 1;
                    let previous = *sizes.last().ok_or(Error::InvalidBlock)? as i64;
                    sizes.push(
                        usize::try_from(previous + encoded as i64 - bias)
                            .map_err(|_| Error::InvalidBlock)?,
                    );
                }
                let used_sum: usize = sizes.iter().sum();
                sizes.push(
                    data.len()
                        .checked_sub(cursor + used_sum)
                        .ok_or(Error::InvalidBlock)?,
                );
            }
        }
        _ => return Err(Error::UnsupportedLacing),
    };
    let payload = &data[cursor..];
    if sizes.iter().sum::<usize>() != payload.len() {
        return Err(Error::InvalidBlock);
    }
    let mut offset: usize = 0;
    let mut out = Vec::with_capacity(sizes.len());
    for size in sizes {
        let end = offset.checked_add(size).ok_or(Error::InvalidBlock)?;
        out.push((
            track,
            relative,
            keyframe.unwrap_or(flags & 0x80 != 0),
            flags & 0x08 != 0,
            payload[offset..end].to_vec(),
        ));
        offset = end;
    }
    Ok(out)
}

fn elements(data: &[u8]) -> Result<impl Iterator<Item = Result<Element<'_>, Error>>, Error> {
    let mut result = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let (id, id_len) = vint(data, pos, true)?;
        pos += id_len;
        let (size, size_len) = vint(data, pos, false)?;
        pos += size_len;
        let size = usize::try_from(size).map_err(|_| Error::InvalidElement)?;
        let unknown_size = size as u64 == (1u64 << (7 * size_len)) - 1;
        let size = if unknown_size {
            data.len() - pos
        } else {
            usize::try_from(size).map_err(|_| Error::InvalidElement)?
        };
        let end = pos.checked_add(size).ok_or(Error::InvalidElement)?;
        let body = data.get(pos..end).ok_or(Error::Truncated)?;
        result.push(Ok(Element { id, data: body }));
        pos = end;
        if unknown_size {
            break;
        }
    }
    Ok(result.into_iter())
}

fn vint(data: &[u8], pos: usize, keep_marker: bool) -> Result<(u64, usize), Error> {
    let first = *data.get(pos).ok_or(Error::Truncated)?;
    if first == 0 {
        return Err(Error::InvalidVint);
    }
    let len = first.leading_zeros() as usize + 1;
    if len > 8 {
        return Err(Error::InvalidVint);
    }
    let slice = data.get(pos..pos + len).ok_or(Error::Truncated)?;
    let mut value = if keep_marker {
        first as u64
    } else {
        (first & ((0xffu16 >> len) as u8)) as u64
    };
    for b in &slice[1..] {
        value = (value << 8) | *b as u64;
    }
    Ok((value, len))
}

fn uint(data: &[u8]) -> Result<u64, Error> {
    if data.is_empty() || data.len() > 8 {
        return Err(Error::InvalidElement);
    }
    Ok(data.iter().fold(0u64, |v, b| (v << 8) | *b as u64))
}

fn text(data: &[u8]) -> Result<String, Error> {
    String::from_utf8(data.to_vec()).map_err(|_| Error::InvalidElement)
}

fn float(data: &[u8]) -> Result<f64, Error> {
    match data.len() {
        4 => Ok(f32::from_bits(u32::from_be_bytes(data.try_into().unwrap())) as f64),
        8 => Ok(f64::from_bits(u64::from_be_bytes(data.try_into().unwrap()))),
        _ => Err(Error::InvalidElement),
    }
}

pub fn codec_tracks(tracks: &[Track]) -> HashMap<TrackKind, Vec<&Track>> {
    let mut result: HashMap<TrackKind, Vec<&Track>> = HashMap::new();
    for track in tracks {
        result.entry(track.kind).or_default().push(track);
    }
    result
}
