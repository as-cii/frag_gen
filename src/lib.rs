#![feature(stdsimd)]

extern crate smallvec;

use smallvec::SmallVec;
use std::cmp::Ordering;
use std::fmt;
use std::iter;
use std::simd::{m16x16, m1x16, u16x16};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CompositeId {
    entries: SmallVec<[Id; 1]>,
}

#[derive(Copy, Clone)]
struct Id {
    entries: u16x16,
    len: u8,
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

impl CompositeId {
    fn new(value: u16) -> Self {
        let mut entries = SmallVec::new();
        entries.push(Id::new(value));
        Self { entries }
    }

    fn between_with_max(a: &Self, b: &Self, max: u16) -> Self {
        debug_assert!(a < b);
        let a = a.entries.iter().cloned().chain(iter::repeat(Id::new(0)));
        let b = b.entries.iter().cloned().chain(iter::repeat(Id::new(max)));
        let mut entries = SmallVec::new();
        for (a, b) in a.zip(b) {
            if a == b {
                entries.push(a);
            } else if let Ok(middle) = Id::between_with_max(a, b, max) {
                entries.push(middle);
                break;
            } else {
                entries.push(a);
            }
        }
        Self { entries }
    }
}

impl Id {
    fn new(value: u16) -> Self {
        Self {
            entries: u16x16::splat(value),
            len: 1,
        }
    }

    fn between_with_max(a: Self, b: Self, max: u16) -> Result<Self, ()> {
        debug_assert!(a < b);
        let a = MASKS[a.len as usize - 1].select(u16x16::splat(0), a.entries);
        let b = MASKS[b.len as usize - 1].select(u16x16::splat(max), b.entries);
        let middle = a + ((b - a) / 2);
        Ok(Id {
            entries: middle,
            len: compute_len(middle.gt(a))?,
        })
    }

    fn entries(&self) -> u16x16 {
        MASKS[self.len as usize - 1].select(u16x16::splat(0), self.entries)
    }
}

fn compute_len(mask: m16x16) -> Result<u8, ()> {
    for i in 0_u8..16_u8 {
        if mask.extract(i as usize) {
            return Ok(i + 1);
        }
    }
    Err(())
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
            list.entry(&self.entries.extract(i as usize));
        }
        list.finish()
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use self::rand::{Rng, SeedableRng, StdRng};
    use super::*;

    #[test]
    fn test_composite_id_generation() {
        for seed in 0..50 {
            println!("Seed {:?}", seed);
            const MAX_VALUE: u16 = 2;
            let mut rng = StdRng::from_seed(&[seed]);
            let mut ids = vec![CompositeId::new(0), CompositeId::new(MAX_VALUE)];
            for _i in 0..200 {
                let index = rng.gen_range::<usize>(1, ids.len());
                let middle = {
                    let left = &ids[index - 1];
                    let right = &ids[index];
                    CompositeId::between_with_max(left, right, MAX_VALUE)
                };
                ids.insert(index, middle);

                let mut sorted_ids = ids.clone();
                sorted_ids.sort();
                assert_eq!(ids, sorted_ids);
            }
        }
    }

    #[test]
    fn test_primitive_id_generation() {
        for seed in 0..100 {
            println!("Seed {:?}", seed);
            const MAX_VALUE: u16 = 4;
            let mut rng = StdRng::from_seed(&[seed]);
            let mut ids = vec![Id::new(0), Id::new(MAX_VALUE)];
            for _i in 0..50 {
                let index = rng.gen_range::<usize>(1, ids.len());
                let left = ids[index - 1];
                let right = ids[index];
                ids.insert(index, Id::between_with_max(left, right, MAX_VALUE).unwrap());

                let mut sorted_ids = ids.clone();
                sorted_ids.sort();
                assert_eq!(ids, sorted_ids);
            }
        }
    }
}
