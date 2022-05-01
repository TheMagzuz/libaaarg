use std::time::Duration;
use rodio::{source::{Source, SamplesConverter}, buffer::SamplesBuffer};

pub mod encoding;

pub fn alias<I: Source>(samples: SamplesConverter<I, f32>, duration: Duration, factor: usize) -> SamplesBuffer<f32> where <I as Iterator>::Item: rodio::Sample {
    let aliased = samples.take_duration(duration*factor as u32).step_by(factor);
    SamplesBuffer::new(2, 44100, aliased.collect::<Vec<f32>>())
} 
