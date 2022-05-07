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

pub mod encoding;
pub mod blocks;
