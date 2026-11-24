//! MPEG Layer III decoder plugin for the TVP Sound System.

mod decoder;

#[cfg(windows)]
mod tss;

pub use decoder::AudioFormat;
