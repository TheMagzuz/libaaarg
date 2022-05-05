use std::time::Duration;
use rand::prelude::*;
use rodio::{source::{Source, SamplesConverter}, buffer::SamplesBuffer};

pub mod encoding;

#[derive(Clone)]
pub struct AliasingParams {
    pub factor: usize,
    pub target_duration: Duration,
    pub factor_variation: usize,
}

impl Default for AliasingParams {
    fn default() -> Self {
        Self {
            factor: 1,
            target_duration: Duration::from_secs(1),
            factor_variation: 0,
        }
    }
}


impl AliasingParams {
    pub fn from_secs(target_duration_secs: f32, factor: usize, variation: usize) -> Self {
        Self {
            factor,
            target_duration: Duration::from_secs_f32(target_duration_secs),
            factor_variation: variation,
        }
    }
}

pub fn alias<I: Source>(samples: SamplesConverter<I, f32>, params: &AliasingParams) -> SamplesBuffer<f32> where <I as Iterator>::Item: rodio::Sample {
    let duration = params.target_duration;
    let factor = params.factor;
    let variation = params.factor_variation as isize;
    let aliased = if variation == 0 {
        // For some reason you need to multiply the duration by 4 to get the correct duration.
        // Don't ask me why...
        samples.take_duration(duration*factor as u32*4).step_by(factor).collect::<Vec<f32>>()
    } else {
        let sample_count = (duration.as_secs() * samples.sample_rate() as u64) as usize;
        let samples: Vec<_> = samples.take_duration(duration*factor as u32).collect();
        let mut v = Vec::with_capacity(sample_count);
        let mut i = 0;
        let mut rng = rand::thread_rng();
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
    SamplesBuffer::new(2, 44100, aliased)
} 
