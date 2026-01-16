#![allow(clippy::useless_conversion)]

use std::{
    fmt::{Debug, Display},
    mem::{size_of, size_of_val},
};

use binrw::binrw;
use bit_reverse::ParallelReverse;
use bit_vec::BitVec;
use itertools::Itertools;
use to_binary::BinaryString;

use super::sequence_number::SequenceNumber;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[binrw]
pub struct SequenceNumberSet {
    base: SequenceNumber,
    num_bits: u32,
    #[br(count = SequenceNumberSet::calculate_num_word(num_bits))]
    bitmaps: Vec<u32>,
}

impl SequenceNumberSet {
    pub fn new(base: SequenceNumber, set: &[SequenceNumber]) -> Self {
        // TODO: handle properly the case where set.len() could be bigger than 256
        assert!(set.len() <= 256);
        let num_bits = SequenceNumberSet::calculate_num_bits(base, set);
        let bitmaps = SequenceNumberSet::calculate_bitmaps(base, num_bits, set);

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
        if self.num_bits > 0 {
            size += size_of::<u32>() * self.bitmaps.len();
        }
        size
    }

    pub fn get_base(&self) -> SequenceNumber {
        self.base
    }

    pub fn get_set(&self) -> Vec<SequenceNumber> {
        let bitmaps: Vec<u8> = self
            .bitmaps
            .iter()
            .flat_map(|word| word.to_be_bytes())
            .collect();
        let bitmaps = BitVec::from_bytes(&bitmaps);

        (0..self.num_bits)
            .into_iter()
            .filter_map(|offset| {
                if bitmaps.get(offset as usize).unwrap() {
                    Some(self.base + offset as i64)
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

    fn calculate_num_bits(base: SequenceNumber, set: &[SequenceNumber]) -> u32 {
        if set.is_empty() {
            return 0;
        }
        let highest_seq = set.iter().max().cloned().unwrap();
        ((highest_seq - base).0 + 1) as u32
    }

    fn calculate_bitmaps(base: SequenceNumber, num_bits: u32, set: &[SequenceNumber]) -> Vec<u32> {
        if set.is_empty() {
            return Vec::default();
        }
        let set = set.iter().sorted().collect::<Vec<_>>();

        let num_words = SequenceNumberSet::calculate_num_word(num_bits);
        let mut bitvec = BitVec::from_elem(num_words * 32, false);
        for i in base.0..(base.0 + num_bits as i64) {
            if set.contains(&&SequenceNumber(i)) {
                bitvec.set((i - base.0) as usize, true);
            }
        }

        let bitvec = bitvec
            .to_bytes()
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .as_slice()
            .chunks(4)
            .map(<[u8; 4] as TryFrom<&[u8]>>::try_from)
            .map(Result::unwrap)
            .map(u32::from_be_bytes)
            .collect::<Vec<_>>();

        bitvec
    }
}

impl Debug for SequenceNumberSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("0                                                           32\n")?;
        f.write_str("+-----------------------------------------------------------+\n")?;
        f.write_str(&format!(
            "|            base: {:0>4}                                     |\n",
            self.base
        ))?;
        f.write_str("+                                                           +\n")?;
        f.write_str("|                                                           |\n")?;
        f.write_str("+-----------------------------------------------------------+\n")?;
        f.write_str(&format!(
            "|        num_bits: {:0>4}                                     |\n",
            self.num_bits
        ))?;
        for (i, word) in self.bitmaps.iter().enumerate() {
            f.write_str("+-----------------------------------------------------------+\n")?;
            let word: [u8; 4] = word.to_ne_bytes();
            let word = BinaryString::from(&word[..]).add_spaces().unwrap();
            f.write_str(&format!("|      bitmap[{i}]: {word}       |\n"))?;
        }
        f.write_str("+-----------------------------------------------------------+\n")?;
        Ok(())
    }
}

impl Display for SequenceNumberSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = format!("<{},[{}]>", self.base, self.get_set().iter().join(","));
        f.write_str(&str)?;
        Ok(())
    }
}

#[cfg(test)]
mod sequence_number_set_tests {
    use rstest::*;
    use rstest_reuse::{self, *};

    use super::SequenceNumberSet;
    use crate::types::SequenceNumber;

    #[rstest]
    #[case(SequenceNumber(1234), &[], 0, &[], 12)]
    #[case(SequenceNumber(1234), &[SequenceNumber(1234)], 1, &[0b_0000_0000_0000_0000_0000_0000_0000_0001], 16)]
    #[case(SequenceNumber(1234), &[SequenceNumber(1235)], 2, &[0b_0000_0000_0000_0000_0000_0000_0000_0010], 16)]
    #[case(SequenceNumber(1234), &[SequenceNumber(1265)], 32, &[0b_1000_0000_0000_0000_0000_0000_0000_0000], 16)]
    #[case(SequenceNumber(1234), &[SequenceNumber(1266)], 33, &[0b_0000_0000_0000_0000_0000_0000_0000_0000, 0b_0000_0000_0000_0000_0000_0000_0000_0001], 20)]
    fn test_frag_num_set(
        #[case] base: SequenceNumber,
        #[case] set: &[SequenceNumber],
        #[case] expected_num_bits: u32,
        #[case] expected_bitmap: &[u32],
        #[case] expected_size: usize,
    ) {
        let seq_num_set = SequenceNumberSet::new(base, set);
        assert_eq!(expected_num_bits, seq_num_set.num_bits);
        assert_eq!(expected_bitmap, seq_num_set.bitmaps);
        assert_eq!(expected_size, seq_num_set.size());
    }
}
