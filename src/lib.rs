#![warn(missing_docs)]
//! A library for mangling audio in different ways
//!
//! This library provides the [`alias`] function (subject to be renamed), which will mangle a given
//! piece of audio in various different ways.
//!
//! # A warning
//! **Protect your hearing.** Whenever working with audio, make sure to **not** wear your
//! headphones before doing something that might output audio, as it may be unexpectedly loud and
//! damage your hearing.
//!
//! # Quickstart
//! ```
//! use std::fs::File;
//! use std::time::Duration;
//! use std::io::BufReader;
//! use rodio::{Decoder, OutputStream, Sink};
//! use libaaarg::{self, AliasingParams};
//!
//! // Get a output stream handle to the default physical sound device
//! let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//! // Create a sink to output sound to
//! let sink = Sink::try_new(&stream_handle).unwrap();
//! // Load a sound from a file, using a path relative to Cargo.toml
//! let file = BufReader::new(File::open("examples/music.ogg").unwrap());
//! // Decode that sound file into a source
//! let source = Decoder::new(file).unwrap();
//!
//! // Convert the source to an iterator of floating point values.
//! let samples = source.convert_samples::<f32>();
//! // Process the sound, speeding it up by 100x, and limiting the duration of the output sound to
//! // 5 seconds
//! let aliased = libaaarg::alias(samples, &AliasingParams {
//!     factor: 100,
//!     target_duration: Duration::from_secs(5),
//! });
//!
//! // Play the sound directly on the device
//! sink.append(aliased);
//! ```

use std::time::Duration;
use std::ops::RangeInclusive;
use rand::prelude::*;
use rodio::{source::{Source, SamplesConverter}, buffer::SamplesBuffer};

pub mod encoding;
pub mod blocks;

#[derive(Clone)]
/// The parameters determining how audio will be processed
pub struct AliasingParams {
    /// How long the output sample should last. Note that this is only a maximum, if the output of
    /// the processing is shorter than this duration, this value will simply be ignored.
    pub target_duration: Duration,
    /// A range determining how many stutters are to be placed throughout the sample
    pub stutter_count: RangeInclusive<u16>,
    /// A range determining how long each stutter lasts
    pub stutter_duration: RangeInclusive<Duration>,
    /// A range determining how long the individual pieces of each stutter last. Note that this
    /// value is only randomized at the start of each stutter, although randomizing it throughout
    /// the stutter is planned.
    pub stutter_piece_length: RangeInclusive<Duration>,
}

impl Default for AliasingParams {
    /// This value should always represent an identity function, ie. passing this to the [`alias`]
    /// function should give the input back.
    fn default() -> Self {
        Self {
            target_duration: Duration::from_secs(1),
            stutter_count: 0..=0,
            stutter_duration: Duration::ZERO..=Duration::ZERO,
            stutter_piece_length: Duration::ZERO..=Duration::ZERO,
        }
    }
}


impl AliasingParams {
    /// Create aliasing parameters from a duration, given in seconds, a factor and a factor
    /// variation
    #[deprecated]
    pub fn from_secs(target_duration_secs: f32) -> Self {
        Self {
            target_duration: Duration::from_secs_f32(target_duration_secs),
            ..Default::default()
        }
    }
}

/// Process the given audio
///
/// Processes the audio given by `samples`, using the parameters described by `params`.
///
/// # Overview
/// Currently, the function does the following:
/// 1. Add stutters to the sound, by picking a random point in the sound, and repeating the next
///    [`params.stutter_piece_length`][AliasingParams::stutter_piece_length] samples until
///    [`params.stutter_duration`][AliasingParams::stutter_duration] samples have been repeated.
///    This process will be repeated [`params.stutter_count`][AliasingParams::stutter_count] times.
///    Note that for values that are a range, a random value in the range will be generated.
pub fn alias<I: Source>(source: SamplesConverter<I, f32>, params: &AliasingParams) -> SamplesBuffer<f32> where <I as Iterator>::Item: rodio::Sample {
    // TODO: This should maybe be separated into several functions, that handle each step of the
    // processing
    let mut rng = rand::thread_rng();
    let sample_rate = source.sample_rate();

    let duration_to_samples = |d: Duration| {
        (d.as_secs_f32() * sample_rate as f32).floor() as u64
    };

    let mut source = source.collect::<Vec<f32>>();

    if *params.stutter_count.end() > 0u16 {
        let stutter_count = rng.gen_range(params.stutter_count.clone());
        for _i in 0..stutter_count {
            let stutter_duration = duration_to_samples(rng.gen_range(params.stutter_duration.clone()));
            let stutter_piece_length = duration_to_samples(rng.gen_range(params.stutter_piece_length.clone()));
            // We want to avoid cutting stutters short, to ensure we don't go out of bounds.
            let stutter_location = rng.gen_range(0..(source.len() as u64 - stutter_duration));

            let mut stuttered_samples = 0;
            while stuttered_samples < stutter_duration {
                // All this casting nonsense might cause problems on 32-bit devices, but I'm going
                // to leave that as a problem for future me.
                let start = (stutter_location + stuttered_samples) as usize;
                let stutter_piece_length = if start + stutter_piece_length as usize > source.len() {
                    start - source.len()
                } else {
                    stutter_piece_length as usize
                };
                source.copy_within(stutter_location as usize..stutter_location as usize+stutter_piece_length, start);
                stuttered_samples += stutter_piece_length as u64;
            }
        }
    }
    SamplesBuffer::new(2, 44100, source)
}
