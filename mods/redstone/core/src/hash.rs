//! A cheap hash for positions: the standard one is made to resist attacks, which costs.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
pub struct PosHasher(u64);

impl Hasher for PosHasher {
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.mix(*b as u64);
        }
    }
    fn write_i32(&mut self, i: i32) {
        self.mix(i as u32 as u64);
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

impl PosHasher {
    fn mix(&mut self, v: u64) {
        self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
}

pub type PosMap<K, V> = HashMap<K, V, BuildHasherDefault<PosHasher>>;
pub type PosSet<K> = HashSet<K, BuildHasherDefault<PosHasher>>;
