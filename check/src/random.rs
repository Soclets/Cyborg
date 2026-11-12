//! A small seeded random source, so every generated program can be made again from its seed.

/// `SplitMix64`: tiny, fast, and the same on every machine.
pub struct Random {
    state: u64,
}

impl Random {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }

    /// A number from `low` to `high`, both included.
    pub fn between(&mut self, low: i128, high: i128) -> i128 {
        if high <= low {
            return low;
        }
        let span = u128::try_from(high - low)
            .unwrap_or(u128::MAX)
            .saturating_add(1);
        let pick = u128::from(self.next()) % span;
        low + i128::try_from(pick).unwrap_or(0)
    }

    /// One of a slice's items.
    pub fn pick<'items, T>(&mut self, items: &'items [T]) -> Option<&'items T> {
        if items.is_empty() {
            return None;
        }
        let count = u64::try_from(items.len()).unwrap_or(u64::MAX);
        let index = usize::try_from(self.next() % count).unwrap_or(0);
        items.get(index)
    }

    /// True this often, out of a hundred.
    pub fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}
