use std::time::Duration;
use std::ops::RangeInclusive;
use rand::prelude::*;
use rodio::{source::{Source, SamplesConverter}, buffer::SamplesBuffer};

pub mod encoding;

#[derive(Clone)]
pub struct AliasingParams {
    pub factor: usize,
    pub target_duration: Duration,
    pub factor_variation: usize,
    pub stutter_count: RangeInclusive<u16>,
    pub stutter_duration: RangeInclusive<Duration>,
    pub stutter_piece_length: RangeInclusive<Duration>,
}

impl Default for AliasingParams {
    fn default() -> Self {
        Self {
            factor: 1,
            target_duration: Duration::from_secs(1),
            factor_variation: 0,
            stutter_count: 0..=0,
            stutter_duration: Duration::ZERO..=Duration::ZERO,
            stutter_piece_length: Duration::ZERO..=Duration::ZERO,
        }
    }
}


impl AliasingParams {
    pub fn from_secs(target_duration_secs: f32, factor: usize, variation: usize) -> Self {
        Self {
            factor,
            target_duration: Duration::from_secs_f32(target_duration_secs),
            factor_variation: variation,
            ..Default::default()
        }
    }
}

pub fn alias<I: Source>(samples: SamplesConverter<I, f32>, params: &AliasingParams) -> SamplesBuffer<f32> where <I as Iterator>::Item: rodio::Sample {
    // TODO: This should maybe be separated into several functions, that handle each step of the
    // processing
    let duration = params.target_duration;
    let factor = params.factor;
    let variation = params.factor_variation as isize;
    let mut rng = rand::thread_rng();
    let sample_rate = samples.sample_rate();

    let duration_to_samples = |d: Duration| {
        (d.as_secs_f32() * sample_rate as f32).floor() as u64
    };

    let mut aliased = if variation == 0 {
        // For some reason you need to multiply the duration by 4 to get the correct duration.
        // Don't ask me why...
        samples.take_duration(duration*factor as u32*4).step_by(factor).collect::<Vec<f32>>()
    } else {
        let sample_count = duration_to_samples(duration) as usize;
        let samples: Vec<_> = samples.take_duration(duration*factor as u32).collect();
        let mut v = Vec::with_capacity(sample_count);
        let mut i = 0;
        let mut collected = 0;

        while collected < sample_count {
            if let Some(s) = samples.get(i) {
                v.push(*s);
            } else {
                break;
            };
            collected += 1;
            i += (rng.gen_range(-variation..=variation) + factor as isize).max(0) as usize;
        }
        v
    };
    if *params.stutter_count.end() > 0u16 {
        let stutter_count = rng.gen_range(params.stutter_count.clone());
        for _i in 0..stutter_count {
            let stutter_duration = duration_to_samples(rng.gen_range(params.stutter_duration.clone()));
            let stutter_piece_length = duration_to_samples(rng.gen_range(params.stutter_piece_length.clone()));
            // We want to avoid cutting stutters short, to ensure we don't go out of bounds.
            let stutter_location = rng.gen_range(0..(aliased.len() as u64 - stutter_duration));

            let mut stuttered_samples = 0;
            while stuttered_samples < stutter_duration {
                // All this casting nonsense might cause problems on 32-bit devices, but I'm going
                // to leave that as a problem for future me.
                let start = (stutter_location + stuttered_samples) as usize;
                let stutter_piece_length = if start + stutter_piece_length as usize > aliased.len() {
                    start - aliased.len()
                } else {
                    stutter_piece_length as usize
                };
                aliased.copy_within(stutter_location as usize..stutter_location as usize+stutter_piece_length, start);
                stuttered_samples += stutter_piece_length as u64;
            }
        }
    }
    SamplesBuffer::new(2, 44100, aliased)
}
