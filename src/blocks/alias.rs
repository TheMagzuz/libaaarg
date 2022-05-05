use std::time::Duration;
use rand::prelude::*;

use rodio::{buffer::SamplesBuffer, Sample};

use super::SignalBlock;

/// A block that speeds up the input audio.
///
/// This can be used to create aliasing artifacts, hence the name.
///
/// This block speeds up the audio by taking every `f+v` samples from the output audio, where `f` is
/// [`factor`][Self::factor], and `v` is a random integer in the range
/// `-pv..=pv`, where `pv` is [`factor_variation`][Self::factor_variation].
/// Note that a new value for `v` is generated each time a sample is taken.
pub struct AliasBlock {
    /// How much the audio will be sped up.
    /// A factor of 1 will not speed the audio up at all, while a factor of 2 will double the
    /// speed.
    /// Setting this to larger values will create aliasing artifacts, which can sound pretty
    /// interesting.
    pub factor: usize,
    /// How much the `factor` can vary at each sampling point. See the documentation for [the struct itself][Self] for more details.
    pub factor_variation: usize,
    /// How long the output sample should last. Note that this is only a maximum, if the output of
    /// the processing is shorter than this duration, this value will simply be ignored.
    pub target_duration: Duration,

}

impl Default for AliasBlock {
    fn default() -> Self {
        Self {
            factor: 1,
            factor_variation: 0,
            target_duration: Duration::from_secs(1),
        }
    }
}


impl SignalBlock for AliasBlock {
    fn process<T, S>(&self, source: T) -> SamplesBuffer<S>
    where S: Sample, T: rodio::Source<Item = S> {
        let sample_rate = source.sample_rate();
        let mut rng = rand::thread_rng();
        let variation = self.factor_variation as isize;

        let aliased = if self.factor_variation == 0 {
            // For some reason you need to multiply the duration by 4 to get the correct duration.
            // Don't ask me why...
            source.take_duration(self.target_duration*self.factor as u32*4).step_by(self.factor).collect::<Vec<S>>()
        } else {
            let sample_count = (self.target_duration.as_secs_f32() * sample_rate as f32).floor() as usize;
            let samples: Vec<_> = source.take_duration(self.target_duration*self.factor as u32).collect();
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
                i += (rng.gen_range(-variation..=variation) + self.factor as isize).max(0) as usize;
            }
            v
        };
        SamplesBuffer::new(2, 44100, aliased)
    }
}
