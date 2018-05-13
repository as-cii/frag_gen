#![feature(stdsimd)]

use std::cmp::Ordering;
use std::fmt;
use std::simd::{m16x16, m1x16, u16x16};

#[derive(Copy, Clone)]
struct Id {
    entries: u16x16,
    len: usize,
}

static MASKS: [m1x16; 16] = [
    m1x16::new(
        false, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true,
    ),
    m1x16::new(
        false, false, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true,
    ),
    m1x16::new(
        false, false, false, true, true, true, true, true, true, true, true, true, true, true,
        true, true,
    ),
    m1x16::new(
        false, false, false, false, true, true, true, true, true, true, true, true, true, true,
        true, true,
    ),
    m1x16::new(
        false, false, false, false, false, true, true, true, true, true, true, true, true, true,
        true, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, true, true, true, true, true, true, true, true,
        true, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, false, true, true, true, true, true, true, true,
        true, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, false, false, true, true, true, true, true, true,
        true, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, false, false, false, true, true, true, true,
        true, true, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, false, false, false, false, true, true, true,
        true, true, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, false, false, false, false, false, true, true,
        true, true, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, false, false, false, false, false, false, true,
        true, true, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        true, true, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, true, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, true,
    ),
    m1x16::new(
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false,
    ),
];

impl Id {
    fn new(value: u16) -> Self {
        Self {
            entries: u16x16::splat(value),
            len: 1,
        }
    }

    fn between_with_max(a: Self, b: Self, max: u16) -> Self {
        debug_assert!(a < b);
        let a = MASKS[a.len - 1].select(u16x16::splat(0), a.entries);
        let b = MASKS[b.len - 1].select(u16x16::splat(max), b.entries);
        let middle = a + ((b - a) / 2);
        Id {
            entries: middle,
            len: compute_len(middle.gt(a)),
        }
    }

    fn entries(&self) -> u16x16 {
        MASKS[self.len - 1].select(u16x16::splat(0), self.entries)
    }
}

#[cfg(target_arch = "x86_64")]
fn compute_len(mask: m16x16) -> usize {
    use std::arch::x86_64::_mm256_movemask_epi8;
    use std::simd::IntoBits;
    unsafe {
        let mask = _mm256_movemask_epi8(mask.into_bits());
        (mask.trailing_zeros() / 2) as usize + 1
    }
}

#[cfg(target_arch = "wasm32")]
fn compute_len(mask: m16x16) -> usize {
    for i in 0..16 {
        if mask.extract(i) {
            return i + 1;
        }
    }
    panic!("Length exceeded 16 elements")
}

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.entries().eq(other.entries()).all()
    }
}

impl Eq for Id {}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.entries().partial_cmp(&other.entries())
    }
}

impl Ord for Id {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut list = fmt.debug_list();
        for i in 0..self.len {
            list.entry(&self.entries.extract(i));
        }
        list.finish()
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;

    #[test]
    fn test_id_generation() {
        for seed in 0..100 {
            use self::rand::{Rng, SeedableRng, StdRng};

            const MAX_VALUE: u16 = 4;
            let mut rng = StdRng::from_seed(&[seed]);
            let mut ids = vec![Id::new(0), Id::new(MAX_VALUE)];
            for _i in 0..50 {
                let index = rng.gen_range::<usize>(1, ids.len());
                let left = ids[index - 1];
                let right = ids[index];
                ids.insert(index, Id::between_with_max(left, right, MAX_VALUE));

                let mut sorted_ids = ids.clone();
                sorted_ids.sort();
                assert_eq!(ids, sorted_ids);
            }
        }
    }
}
