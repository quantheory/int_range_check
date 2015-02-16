
use std::num::Int;
use std::cmp::{min, max};

use self::MergeResult::*;

#[derive(Copy,Debug,Eq,PartialEq)]
struct MergeRange<T: Int> {
    start: T,
    end: T,
}

#[derive(Copy,Debug,Eq,PartialEq)]
enum MergeResult<T: Int> {
    Separate,
    Adjacent(MergeRange<T>),
    Overlap(MergeRange<T>, MergeRange<T>),
}

impl<T: Int> MergeRange<T> {
    fn from_range(start: T, end: T) -> Self {
        assert!(start <= end);
        MergeRange{start: start, end: end}
    }
    fn to_range(self) -> (T, T) {
        (self.start, self.end)
    }
    fn from_range_to(end: T) -> Self {
        MergeRange::from_range(<T as Int>::min_value(), end)
    }
    fn from_range_from(start: T) -> Self {
        MergeRange::from_range(start, <T as Int>::max_value())
    }
    fn range_full() -> Self {
        MergeRange::from_range(<T as Int>::min_value(), <T as Int>::max_value())
    }
    fn concatenate(self, other: Self) -> Option<Self> {
        if self.end < (<T as Int>::max_value()) &&
            self.end + <T as Int>::one() == other.start {
                Some(MergeRange{start: self.start, end: other.end})
            } else {
                None
            }
    }
    fn merge(self, other: Self) -> MergeResult<T> {
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
        assert_eq!(MergeRange::range_full().to_range(),
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
        assert_eq!(x.merge(y), Adjacent(MergeRange::range_full()));
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
