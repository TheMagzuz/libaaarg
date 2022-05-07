use std::ops::RangeInclusive;
use std::time::Duration;

use rodio::{Sample, source::Source, buffer::SamplesBuffer};
use rand::prelude::*;

use super::SignalBlock;

/// Add stutters to the sound, by picking a random point in the sound, and repeating the next
/// [`params.stutter_piece_length`][AliasingParams::stutter_piece_length] samples until
/// [`params.stutter_duration`][AliasingParams::stutter_duration] samples have been repeated.
/// This process will be repeated [`params.stutter_count`][AliasingParams::stutter_count] times.
/// Note that for values that are a range, a random value in the range will be generated.
pub struct StutterBlock {
    /// A range determining how many stutters are to be placed throughout the sample
    pub stutter_count: RangeInclusive<u16>,
    /// A range determining how long each stutter lasts
    pub stutter_duration: RangeInclusive<Duration>,
    /// A range determining how long the individual pieces of each stutter last. Note that this
    /// value is only randomized at the start of each stutter, although randomizing it throughout
    /// the stutter is planned.
    pub stutter_piece_length: RangeInclusive<Duration>,
}

impl Default for StutterBlock {
    fn default() -> Self {
        Self {
            stutter_count: 0..=0,
            stutter_duration: Duration::ZERO..=Duration::ZERO,
            stutter_piece_length: Duration::ZERO..=Duration::ZERO,
        }
    }
}

impl SignalBlock for StutterBlock {
    fn process<T, S>(&self, source: T) -> SamplesBuffer<S> where S: Sample, T: Source<Item = S> {
        let mut rng = rand::thread_rng();
        let sample_rate = source.sample_rate();

        let duration_to_samples = |d: Duration| {
            (d.as_secs_f32() * sample_rate as f32).floor() as u64
        };

        let mut source = source.collect::<Vec<S>>();

        if *self.stutter_count.end() > 0u16 {
            let stutter_count = rng.gen_range(self.stutter_count.clone());
            for _i in 0..stutter_count {
                let stutter_duration = duration_to_samples(rng.gen_range(self.stutter_duration.clone()));
                let stutter_piece_length = duration_to_samples(rng.gen_range(self.stutter_piece_length.clone()));
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
}
