
#![feature(core)]
use std::num::{Int, NumCast, ToPrimitive};
use std::cmp::{min, max};

use self::MergeResult::*;

#[derive(Copy,Debug,Eq,PartialEq)]
struct MergeRange {
    start: u64,
    end: u64,
}

#[derive(Copy,Debug,Eq,PartialEq)]
enum MergeResult {
    Separate,
    Adjacent(MergeRange),
    Overlap(MergeRange, MergeRange),
}

fn encode_as_u64<T: Int>(x: T) -> u64 {
    if <T as Int>::min_value() >= <T as Int>::zero() {
        // The unsigned case is easy.
        x.to_u64().unwrap()
    } else {
        // For the signed case, we convert to i64, then convert that to u64 with
        // a shift upward to keep the domain continuous.
        let x_i64 = x.to_i64().unwrap();
        if x_i64 >= 0 {
            x_i64.to_u64().unwrap() + (1u64 << 63)
        } else {
            (x_i64 - <i64 as Int>::min_value()).to_u64().unwrap()
        }
    }
}

fn decode_from_u64<T: Int>(x: u64) -> T {
    if <T as Int>::min_value() >= <T as Int>::zero() {
        <T as NumCast>::from(x).unwrap()
    } else {
        let x_i64 = if x >= (1u64 << 63) {
            (x - (1u64 << 63)).to_i64().unwrap()
        } else {
            x.to_i64().unwrap() + <i64 as Int>::min_value()
        };
        <T as NumCast>::from(x_i64).unwrap()
    }
}

impl MergeRange {
    fn from_range<T: Int>(start: T, end: T) -> MergeRange {
        assert!(start <= end);
        MergeRange {
            start: encode_as_u64(start),
            end: encode_as_u64(end),
        }
    }
    fn to_range<T: Int>(self) -> (T, T) {
        (decode_from_u64(self.start), decode_from_u64(self.end))
    }
    fn from_range_to<T: Int>(end: T) -> MergeRange {
        MergeRange::from_range(<T as Int>::min_value(), end)
    }
    fn from_range_from<T: Int>(start: T) -> MergeRange {
        MergeRange::from_range(start, <T as Int>::max_value())
    }
    fn range_full<T: Int>() -> MergeRange {
        MergeRange::from_range(<T as Int>::min_value(), <T as Int>::max_value())
    }
    fn concatenate(self, other: MergeRange) -> Option<MergeRange> {
        if self.end < (<u64 as Int>::max_value()) &&
            self.end + 1 == other.start {
                Some(MergeRange{start: self.start, end: other.end})
            } else {
                None
            }
    }
    fn merge(self, other: MergeRange) -> MergeResult {
        // Check for adjacent ranges that can be concatenated.
        match self.concatenate(other) {
            Some(concat) => return Adjacent(concat),
            _ => match other.concatenate(self) {
                Some(concat) => return Adjacent(concat),
                _ => (),
            }
        }
        // Check for overlap in the ranges.
        if self.start <= other.end && other.start <= self.end {
            Overlap(MergeRange{start: min(self.start, other.start),
                               end: max(self.end, other.end)},
                    MergeRange{start: max(self.start, other.start),
                               end: min(self.end, other.end)})
        } else {
            Separate
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::Int;
    use super::MergeRange;
    use super::MergeResult::*;
    #[test]
    fn unsigned_range_merge_range_conversion() {
        assert_eq!(MergeRange::from_range(0u32, 20u32).to_range(),
                   (0u32, 20u32));
    }
    #[test]
    fn signed_range_merge_range_conversion() {
        assert_eq!(MergeRange::from_range(-5i64, 5i64).to_range(),
                   (-5i64, 5i64));
        assert_eq!(MergeRange::from_range(0i32, 0i32).to_range(),
                   (0i32, 0i32));
    }
    #[test]
    fn range_to_merge_range_conversion() {
        assert_eq!(MergeRange::from_range_to(2i8).to_range(),
                   (<i8 as Int>::min_value(), 2i8));
    }
    #[test]
    fn range_from_merge_range_conversion() {
        assert_eq!(MergeRange::from_range_from(2u8).to_range(),
                   (2u8, <u8 as Int>::max_value()));
    }
    #[test]
    fn range_full_merge_range_conversion() {
        assert_eq!(MergeRange::range_full::<i32>().to_range(),
                   (<i32 as Int>::min_value(), <i32 as Int>::max_value()));
    }
    #[test]
    fn merge_separate_ranges() {
        let x = MergeRange::from_range(1i32, 2);
        let y = MergeRange::from_range(4i32, 5);
        assert_eq!(x.merge(y), Separate);
    }
    #[test]
    fn merge_adjacent_ranges() {
        let x = MergeRange::from_range(1i32, 2);
        let y = MergeRange::from_range(3i32, 5);
        assert_eq!(x.merge(y), Adjacent(MergeRange::from_range(1i32, 5)));
        assert_eq!(y.merge(x), x.merge(y));
    }
    #[test]
    fn merge_edge_adjacent_ranges() {
        let x = MergeRange::from_range_to(1u64);
        let y = MergeRange::from_range_from(2u64);
        assert_eq!(x.merge(y), Adjacent(MergeRange::range_full::<u64>()));
        assert_eq!(y.merge(x), x.merge(y));
    }
    #[test]
    fn merge_overlapping_ranges() {
        let x = MergeRange::from_range(-1i8, 2);
        let y = MergeRange::from_range(0i8, 5);
        assert_eq!(x.merge(y), Overlap(MergeRange::from_range(-1i8, 5),
                                       MergeRange::from_range(0i8, 2)));
        assert_eq!(y.merge(x), x.merge(y));
    }
    #[test]
    fn merge_contained_range() {
        let x = MergeRange::from_range(-1i8, 5);
        let y = MergeRange::from_range(0i8, 2);
        assert_eq!(x.merge(y), Overlap(MergeRange::from_range(-1i8, 5),
                                       MergeRange::from_range(0i8, 2)));
        assert_eq!(y.merge(x), x.merge(y));
    }
}
