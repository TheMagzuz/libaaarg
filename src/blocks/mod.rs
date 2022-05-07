//! Signal blocks, that process a given input signal.

use rodio::{Source, Sample, buffer::SamplesBuffer};

mod alias;
mod stutter;

pub use self::alias::AliasBlock;
pub use self::stutter::StutterBlock;

/// A signal block, that can process an audio source in some way.
pub trait SignalBlock {
    /// Process the given `source`, returning a transformed copy of it.
    fn process<T, S>(&self, source: T) -> SamplesBuffer<S> where S: Sample, T: Source<Item = S>;
}
