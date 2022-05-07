//! Signal blocks, that process a given input signal.

use rodio::Source;
use rodio::buffer::SamplesBuffer;

mod alias;
mod stutter;

pub use self::alias::AliasBlock;
pub use self::stutter::StutterBlock;

/// A signal block, that can process an audio source in some way.
pub trait SignalBlock {
    /// Process the given `source`, returning a transformed copy of it.
    // Sadly this method has to be constrained to f32 values, since trait objects cannot have
    // generic methods. See https://doc.rust-lang.org/reference/items/traits.html#object-safety
    fn process(&self, source: Box<dyn Source<Item = f32>>) -> SamplesBuffer<f32>;
}
