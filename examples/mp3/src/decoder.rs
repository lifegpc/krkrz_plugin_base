//! Streaming MP3 decoding and sample-position seeking.

use std::io::{Cursor, Read};
use std::sync::Arc;

use krkrz_plugin_base::com::IStreamWrapper;
use puremp3::{Frame, Mp3Decoder};
use windows::Win32::System::Com::IStream;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u32,
}

/// Playback state used by the TSS COM decoder. The compressed source is kept
/// in memory so seeking can rebuild decoder history, including the MP3 bit
/// reservoir, without requiring IStream::Clone (which TVP streams do not
/// implement).
pub(crate) struct Mp3Playback {
    source: Arc<[u8]>,
    decoder: Mp3Decoder<Cursor<Arc<[u8]>>>,
    frame: Option<Frame>,
    frame_offset: usize,
    format: AudioFormat,
    ended: bool,
}

impl Mp3Playback {
    pub(crate) fn new(stream: IStream) -> Result<Self, ()> {
        let mut stream = IStreamWrapper(stream);
        let mut compressed = Vec::new();
        stream.read_to_end(&mut compressed).map_err(|_| ())?;
        if compressed.is_empty() {
            return Err(());
        }
        Self::from_source(Arc::from(compressed))
    }

    fn from_source(source: Arc<[u8]>) -> Result<Self, ()> {
        let mut decoder = Mp3Decoder::new(Cursor::new(Arc::clone(&source)));
        let frame = decoder.next_frame().map_err(|_| ())?;
        let format = AudioFormat {
            sample_rate: frame.header.sample_rate.hz(),
            channels: frame.header.channels.num_channels() as u32,
        };
        if format.sample_rate == 0 || !(1..=2).contains(&format.channels) {
            return Err(());
        }

        Ok(Self {
            source,
            decoder,
            frame: Some(frame),
            frame_offset: 0,
            format,
            ended: false,
        })
    }

    pub(crate) fn format(&self) -> AudioFormat {
        self.format
    }

    /// Fill `output` with interleaved PCM16 samples and return the number of
    /// sample frames written. A sample frame contains one value per channel.
    pub(crate) fn render(&mut self, output: &mut [i16]) -> Result<usize, ()> {
        let channels = self.format.channels as usize;
        if output.len() % channels != 0 {
            return Err(());
        }
        let requested = output.len() / channels;
        let mut written = 0;

        while written < requested {
            if self.ended {
                break;
            }

            if self.frame.is_none() {
                match self.next_frame() {
                    Some(frame) => {
                        self.frame = Some(frame);
                        self.frame_offset = 0;
                    }
                    None => {
                        self.ended = true;
                        break;
                    }
                }
            }

            let frame = self.frame.as_ref().ok_or(())?;
            if frame.header.sample_rate.hz() != self.format.sample_rate
                || frame.header.channels.num_channels() as u32 != self.format.channels
            {
                return Err(());
            }
            let available = frame.num_samples.saturating_sub(self.frame_offset);
            if available == 0 {
                self.frame = None;
                continue;
            }

            let count = available.min(requested - written);
            for index in 0..count {
                let source_index = self.frame_offset + index;
                let output_index = (written + index) * channels;
                output[output_index] = pcm16(frame.samples[0][source_index]);
                if channels == 2 {
                    output[output_index + 1] = pcm16(frame.samples[1][source_index]);
                }
            }

            self.frame_offset += count;
            written += count;
            if self.frame_offset == frame.num_samples {
                self.frame = None;
            }
        }

        Ok(written)
    }

    /// Reset from the start and decode/discard up to an interleaved sample
    /// frame. Replaying from the beginning preserves MP3 reservoir state.
    pub(crate) fn seek(&mut self, sample_position: u64) -> Result<(), ()> {
        let mut replacement = Self::from_source(Arc::clone(&self.source))?;
        replacement.skip_to(sample_position)?;
        *self = replacement;
        Ok(())
    }

    fn skip_to(&mut self, target: u64) -> Result<(), ()> {
        let mut remaining = target;

        while remaining != 0 {
            if self.frame.is_none() {
                self.frame = self.next_frame();
                self.frame_offset = 0;
            }

            let Some(frame) = self.frame.as_ref() else {
                return Err(());
            };
            if frame.header.sample_rate.hz() != self.format.sample_rate
                || frame.header.channels.num_channels() as u32 != self.format.channels
            {
                return Err(());
            }

            let available = frame.num_samples.saturating_sub(self.frame_offset);
            if available == 0 {
                self.frame = None;
                continue;
            }
            let skip = available.min(remaining.min(usize::MAX as u64) as usize);
            self.frame_offset += skip;
            remaining -= skip as u64;
            if self.frame_offset == frame.num_samples {
                self.frame = None;
            }
        }

        Ok(())
    }

    fn next_frame(&mut self) -> Option<Frame> {
        match self.decoder.next_frame() {
            Ok(frame) => Some(frame),
            // puremp3 reports end-of-stream and malformed trailing data via
            // the same error type. Match its `frames()` iterator behavior and
            // treat either as the end of the decodable stream.
            Err(_) => None,
        }
    }
}

fn pcm16(sample: f32) -> i16 {
    if !sample.is_finite() {
        return 0;
    }
    if sample <= -1.0 {
        i16::MIN
    } else if sample >= 1.0 {
        i16::MAX
    } else {
        (sample * 32768.0) as i16
    }
}
