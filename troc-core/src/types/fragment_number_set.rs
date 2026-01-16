#![allow(clippy::useless_conversion)]

use std::{
    fmt::Display,
    mem::{size_of, size_of_val},
};

use binrw::binrw;
use bit_reverse::ParallelReverse;
use bit_vec::BitVec;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::fragment_number::FragmentNumber;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[binrw]
pub struct FragmentNumberSet {
    base: FragmentNumber,
    num_bits: u32,
    #[brw(little)]
    #[br(count = FragmentNumberSet::calculate_num_word(num_bits))]
    bitmaps: Vec<u32>,
}

impl FragmentNumberSet {
    pub fn new(base: FragmentNumber, set: &[FragmentNumber]) -> Self {
        // TODO: handle properly the case where set.len() could be bigger than 256
        assert!(set.len() <= 256);
        let num_bits = FragmentNumberSet::calculate_num_bits(base, set);
        let bitmaps = FragmentNumberSet::calculate_bitmaps(base, num_bits, set);

        Self {
            base,
            num_bits,
            bitmaps,
        }
    }

    pub fn size(&self) -> usize {
        let mut size = 0;
        size += size_of_val(&self.base);
        size += size_of_val(&self.num_bits);
        size += size_of::<u32>() * self.bitmaps.len();
        size
    }

    pub fn get_base(&self) -> FragmentNumber {
        self.base
    }

    pub fn get_set(&self) -> Vec<FragmentNumber> {
        let bitmaps: Vec<u8> = self
            .bitmaps
            .iter()
            .flat_map(|word| word.to_ne_bytes())
            .map(|block| block.swap_bits())
            .collect();
        let bitmaps = BitVec::from_bytes(&bitmaps);

        (0..self.num_bits)
            .into_iter()
            .filter_map(|offset| {
                if bitmaps.get(offset as usize).unwrap() {
                    Some(self.base + offset as u64)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    fn calculate_num_word(num_bits: u32) -> usize {
        let words = num_bits.div_ceil(32);
        words as usize
    }

    fn calculate_num_bits(base: FragmentNumber, set: &[FragmentNumber]) -> u32 {
        if set.is_empty() {
            return 0;
        }
        let highest_seq = set.iter().max().cloned().unwrap();
        (highest_seq - base).0 + 1
    }

    fn calculate_bitmaps(base: FragmentNumber, num_bits: u32, set: &[FragmentNumber]) -> Vec<u32> {
        let set = set.iter().sorted().collect::<Vec<_>>();

        let num_words = FragmentNumberSet::calculate_num_word(num_bits);
        let mut bitvec = BitVec::from_elem(num_words * 32, false);
        for i in base.0..(base.0 + num_bits) {
            if set.contains(&&FragmentNumber(i)) {
                bitvec.set((i - base.0) as usize, true);
            }
        }

        let bitvec = bitvec
            .to_bytes()
            .into_iter()
            .map(|block| block.swap_bits())
            .collect::<Vec<_>>()
            .into_iter()
            .as_slice()
            .chunks(4)
            .map(<[u8; 4] as TryFrom<&[u8]>>::try_from)
            .map(Result::unwrap)
            .map(u32::from_ne_bytes)
            .collect::<Vec<_>>();

        bitvec
    }
}

impl Display for FragmentNumberSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for seq in self.get_set() {
            f.write_str(&format!("{}, ", seq))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use rstest_reuse::{self, *};

    use super::FragmentNumberSet;
    use crate::types::FragmentNumber;

    #[apply(frag_num_set_template)]
    fn calculate_num_bits(
        given_base: FragmentNumber,
        given_set: &[FragmentNumber],
        given_num_bits: u32,
        _given_bitmaps: Vec<u32>,
    ) {
        let actual_num_bit = FragmentNumberSet::calculate_num_bits(given_base, given_set);
        assert_eq!(actual_num_bit, given_num_bits)
    }

    #[apply(frag_num_set_template)]
    fn calculate_bitmaps(
        given_base: FragmentNumber,
        given_set: &[FragmentNumber],
        given_num_bits: u32,
        given_bitmaps: Vec<u32>,
    ) {
        let actual_bitmaps =
            FragmentNumberSet::calculate_bitmaps(given_base, given_num_bits, given_set);
        assert_eq!(actual_bitmaps, given_bitmaps)
    }

    #[apply(frag_num_set_template)]
    fn get_set(
        given_base: FragmentNumber,
        given_set: &[FragmentNumber],
        given_num_bits: u32,
        given_bitmaps: Vec<u32>,
    ) {
        let frag_num_set = FragmentNumberSet {
            base: given_base,
            num_bits: given_num_bits,
            bitmaps: given_bitmaps.clone(),
        };
        let actual_set = frag_num_set.get_set();
        assert_eq!(actual_set, given_set)
    }

    #[template]
    #[rstest]
    #[case(FragmentNumber(0), &[FragmentNumber(0), FragmentNumber(1), FragmentNumber(2), FragmentNumber(3)], 4, Vec::from_iter([0b_0000_0000_0000_0000_0000_0000_0000_1111]))]
    #[case(FragmentNumber(0), &[FragmentNumber(1), FragmentNumber(2), FragmentNumber(3)], 4, Vec::from_iter([0b_0000_0000_0000_0000_0000_0000_0000_1110]))]
    #[case(FragmentNumber(1), &[FragmentNumber(1), FragmentNumber(2), FragmentNumber(3)], 3, Vec::from_iter([0b_0000_0000_0000_0000_0000_0000_0000_0111]))]
    fn frag_num_set_template(
        #[case] given_base: FragmentNumber,
        #[case] given_set: &[FragmentNumber],
        #[case] given_num_bits: u32,
        #[case] given_bitmaps: Vec<u32>,
    ) {
    }
}
