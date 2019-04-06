use std::ops::{Generator, GeneratorState};
use std::pin::Pin;
use std::marker::Unpin;

// Converts a generator to a iterator.
pub struct GenIter<G>(pub G);

impl<G: Generator + Unpin> Iterator for GenIter<G> {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.0).resume() {
            GeneratorState::Yielded(y) => Some(y),
            GeneratorState::Complete(_) => None,
        }
    }
}

