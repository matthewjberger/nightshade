use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

pub const MASK_WORDS: usize = 4;

pub const MASK_BITS: u32 = (MASK_WORDS * 64) as u32;

const HASH_MIX: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mask([u64; MASK_WORDS]);

impl Mask {
    pub const EMPTY: Self = Self([0; MASK_WORDS]);

    #[inline]
    pub fn bit(index: u32) -> Self {
        let mut mask = Self::EMPTY;
        mask.insert(index);
        mask
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|word| *word == 0)
    }

    #[inline]
    pub fn insert(&mut self, index: u32) {
        debug_assert!(index < MASK_BITS, "slot {index} is past the mask width");
        self.0[(index / 64) as usize] |= 1 << (index % 64);
    }

    #[inline]
    pub fn remove(&mut self, index: u32) {
        self.0[(index / 64) as usize] &= !(1 << (index % 64));
    }

    #[inline]
    pub fn contains_bit(&self, index: u32) -> bool {
        self.0[(index / 64) as usize] & (1 << (index % 64)) != 0
    }

    #[inline]
    pub fn contains(&self, other: Self) -> bool {
        self.0
            .iter()
            .zip(other.0)
            .all(|(mine, wanted)| wanted & !mine == 0)
    }

    #[inline]
    pub fn intersects(&self, other: Self) -> bool {
        self.0
            .iter()
            .zip(other.0)
            .any(|(mine, theirs)| mine & theirs != 0)
    }

    #[inline]
    pub fn count(&self) -> u32 {
        self.0.iter().map(|word| word.count_ones()).sum()
    }

    #[inline]
    pub fn count_below(&self, index: u32) -> u32 {
        let last = (index / 64) as usize;
        let bit = index % 64;
        let whole: u32 = self.0[..last].iter().map(|word| word.count_ones()).sum();
        if bit == 0 {
            whole
        } else {
            whole + (self.0[last] & ((1u64 << bit) - 1)).count_ones()
        }
    }

    #[inline]
    pub fn first(&self) -> Option<u32> {
        self.indices().next()
    }

    #[inline]
    pub fn lowest(count: u32) -> Self {
        let mut mask = Self::EMPTY;
        let full = (count / 64) as usize;
        mask.0[..full].fill(u64::MAX);
        let remainder = count % 64;
        if remainder > 0 {
            mask.0[full] = (1u64 << remainder) - 1;
        }
        mask
    }

    #[inline]
    pub fn highest(count: u32) -> Self {
        let mut mask = Self::EMPTY;
        let full = (count / 64) as usize;
        mask.0[MASK_WORDS - full..].fill(u64::MAX);
        let remainder = count % 64;
        if remainder > 0 {
            mask.0[MASK_WORDS - 1 - full] = !((1u64 << (64 - remainder)) - 1);
        }
        mask
    }

    #[inline]
    pub fn column_of(&self, bit: Self) -> Option<u32> {
        let mut below = 0;
        for (mine, wanted) in self.0.iter().zip(bit.0) {
            if wanted != 0 {
                if mine & wanted == 0 {
                    return None;
                }
                return Some(below + (mine & (wanted - 1)).count_ones());
            }
            below += mine.count_ones();
        }
        None
    }

    #[inline]
    pub fn single_slot(&self) -> Option<u32> {
        let mut found = None;
        for (word, bits) in self.0.iter().enumerate() {
            if *bits == 0 {
                continue;
            }
            if found.is_some() || bits & (bits - 1) != 0 {
                return None;
            }
            found = Some((word as u32) * 64 + bits.trailing_zeros());
        }
        found
    }

    #[inline]
    pub fn indices(&self) -> impl Iterator<Item = u32> + '_ {
        self.0.iter().enumerate().flat_map(|(word, bits)| {
            let base = (word * 64) as u32;
            BitIndices { bits: *bits, base }
        })
    }

    #[inline]
    fn combine(self, other: Self, f: impl Fn(u64, u64) -> u64) -> Self {
        Self(std::array::from_fn(|index| {
            f(self.0[index], other.0[index])
        }))
    }
}

impl std::hash::Hash for Mask {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let folded = self
            .0
            .iter()
            .fold(0u64, |folded, word| (folded ^ word).wrapping_mul(HASH_MIX));
        state.write_u64(folded);
    }
}

struct BitIndices {
    bits: u64,
    base: u32,
}

impl Iterator for BitIndices {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        if self.bits == 0 {
            return None;
        }
        let index = self.bits.trailing_zeros();
        self.bits &= self.bits - 1;
        Some(self.base + index)
    }
}

impl BitAnd for Mask {
    type Output = Self;

    #[inline]
    fn bitand(self, other: Self) -> Self {
        self.combine(other, |mine, theirs| mine & theirs)
    }
}

impl BitOr for Mask {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self {
        self.combine(other, |mine, theirs| mine | theirs)
    }
}

impl Not for Mask {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        Self(std::array::from_fn(|index| !self.0[index]))
    }
}

impl BitOrAssign for Mask {
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl BitAndAssign for Mask {
    #[inline]
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other;
    }
}

impl std::fmt::Debug for Mask {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_set().entries(self.indices()).finish()
    }
}
