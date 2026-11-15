//! A small xorshift generator for drops and bonus offers.

use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Resource)]
pub(crate) struct RandomState(pub(crate) u64);

impl Default for RandomState {
    fn default() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(1, |duration| duration.as_nanos() as u64);
        Self(seed.max(1))
    }
}

impl RandomState {
    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub(crate) fn index(&mut self, length: usize) -> usize {
        (self.next_u64() as usize) % length
    }

    /// A number drawn evenly from 0 to 1, to compare odds against: a draw
    /// that falls under one odds and not another tells which paid out.
    pub(crate) fn roll(&mut self) -> f32 {
        (self.next_u64() as f64 / u64::MAX as f64) as f32
    }
}
