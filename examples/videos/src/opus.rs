//! Opus packet decoding adapter for WebM audio tracks.

pub struct OpusDecoder {
    inner: opus_rs::OpusDecoder,
    channels: usize,
}

impl OpusDecoder {
    pub fn new(channels: usize) -> Result<Self, &'static str> {
        if !(1..=2).contains(&channels) {
            return Err("WebM Opus supports mono or stereo output in this filter");
        }
        Ok(Self {
            inner: opus_rs::OpusDecoder::new(48_000, channels)?,
            channels,
        })
    }

    /// Decode one WebM Opus packet into interleaved float PCM.
    pub fn decode(&mut self, packet: &[u8], frame_size: usize) -> Result<Vec<f32>, &'static str> {
        let samples = frame_size
            .checked_mul(self.channels)
            .ok_or("PCM size overflow")?;
        let mut pcm = vec![0.0; samples];
        let written = self.inner.decode(packet, frame_size, &mut pcm)?;
        pcm.truncate(written * self.channels);
        Ok(pcm)
    }

    pub fn channels(&self) -> usize {
        self.channels
    }
}
