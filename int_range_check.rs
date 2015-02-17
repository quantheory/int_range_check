
#![crate_name(int_range_check)]
#![crate_type="lib"]

use std::cmp::{min, max};
use std::fmt::{self, Display, Formatter};
use std::num::Int;

use self::MergeResult::*;

pub fn uncovered_and_overlapped<T: Int>(ranges: &Vec<IntRange<T>>)
      -> (Vec<IntRange<T>>, Vec<IntRange<T>>) {
    let (range_set, overlap_set) =
        RangeSet::from_vec_with_overlap(&(
            ranges.iter().filter_map(|&x| x.to_merge_range()).collect()
                ));
    let uncovered_set = range_set.complement();
    (uncovered_set.into_vec().iter()
         .map(|&x| IntRange::from_merge_range(x)).collect(),
     overlap_set.into_vec().iter()
         .map(|&x| IntRange::from_merge_range(x)).collect())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntRange<T: Int> {
    Bound(T, T),
    To(T),
    From(T),
    Full,
}

impl<T: Int> IntRange<T> {
    fn to_merge_range(self) -> Option<MergeRange<T>> {
        match self {
            IntRange::Bound(start, end) => if start <= end {
                Some(MergeRange::from_range(start, end))
            } else {
                None
            },
            IntRange::To(end) => Some(MergeRange::from_range_to(end)),
            IntRange::From(start) => Some(MergeRange::from_range_from(start)),
            IntRange::Full => Some(MergeRange::range_full()),
        }
    }
    fn from_merge_range(merge_range: MergeRange<T>) -> Self {
        if merge_range.start > (<T as Int>::min_value()) {
            if merge_range.end < (<T as Int>::max_value()) {
                IntRange::Bound(merge_range.start, merge_range.end)
            } else {
                IntRange::From(merge_range.start)
            }
        } else {
            if merge_range.end < (<T as Int>::max_value()) {
                IntRange::To(merge_range.end)
            } else {
                IntRange::Full
            }
        }
    }
}

impl<T: Display+Int> Display for IntRange<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        let output = match *self {
            IntRange::Bound(start, end) => format!("{}-{}", start, end),
            IntRange::To(end) => format!("{} and below", end),
            IntRange::From(start) => format!("{} and above", start),
            IntRange::Full => format!("{}", "full range")
        };
        formatter.write_str(&*output)
    }
}

impl<T: Display+Int> Display for Vec<IntRange<T>> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        try!(formatter.write_str("["));
        let mut first = true;
        for range in self.iter() {
            if !first {
                try!(formatter.write_fmt(format_args!(", {}", range)));
            } else {
                first = false;
                try!(formatter.write_fmt(format_args!("{}", range)));
            }
        }
        formatter.write_str("]")
    }
}

#[cfg(test)]
mod interface_tests {
    use super::IntRange;
    use super::MergeRange;
    #[test]
    fn bound_convert_merge_range() {
        assert_eq!(IntRange::Bound(2u8, 5u8).to_merge_range(),
                   Some(MergeRange::from_range(2u8, 5u8)));
        assert_eq!(IntRange::Bound(10u8, 10u8).to_merge_range(),
                   Some(MergeRange::from_range(10u8, 10u8)));
    }
    #[test]
    fn empty_bound_convert_merge_range() {
        assert_eq!(IntRange::Bound(5u8, 1u8).to_merge_range(), None);
    }
    #[test]
    fn to_convert_merge_range() {
        assert_eq!(IntRange::To(2u8).to_merge_range(),
                   Some(MergeRange::from_range_to(2u8)));
    }
    #[test]
    fn from_convert_merge_range() {
        assert_eq!(IntRange::From(2u8).to_merge_range(),
                   Some(MergeRange::from_range_from(2u8)));
    }
    #[test]
    fn full_convert_merge_range() {
        assert_eq!(IntRange::Full::<u8>.to_merge_range(),
                   Some(MergeRange::range_full()));
    }
    #[test]
    fn merge_range_convert_bound() {
        let merge_range = MergeRange::from_range(-5i32, -2i32);
        assert_eq!(IntRange::from_merge_range(merge_range),
                   IntRange::Bound(-5i32, -2i32));
    }
    #[test]
    fn merge_range_convert_to() {
        let merge_range = MergeRange::from_range_to(-2i32);
        assert_eq!(IntRange::from_merge_range(merge_range),
                   IntRange::To(-2i32));
    }
    #[test]
    fn merge_range_convert_from() {
        let merge_range = MergeRange::from_range_from(-5i32);
        assert_eq!(IntRange::from_merge_range(merge_range),
                   IntRange::From(-5i32));
    }
    #[test]
    fn merge_range_convert_full() {
        let merge_range = MergeRange::<i32>::range_full();
        assert_eq!(IntRange::from_merge_range(merge_range),
                   IntRange::Full);
    }
    #[test]
    fn display_bound() {
        assert_eq!(format!("{}", IntRange::Bound(8i32, 13)), "8-13")
    }
    #[test]
    fn display_to() {
        assert_eq!(format!("{}", IntRange::To(13i32)), "13 and below")
    }
    #[test]
    fn display_from() {
        assert_eq!(format!("{}", IntRange::From(8i32)), "8 and above")
    }
    #[test]
    fn display_full() {
        assert_eq!(format!("{}", IntRange::Full::<i32>), "full range")
    }
    #[test]
    fn display_vec() {
        let int_range_vec = vec![
            IntRange::To(4u8),
            IntRange::Bound(7u8, 9u8),
            ];
        assert_eq!(format!("{}", int_range_vec), "[4 and below, 7-9]")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RangeSet<T: Int> {
    ranges: Vec<MergeRange<T>>,
}

impl<T: Int> RangeSet<T> {
    fn new() -> Self {
        RangeSet{ranges: Vec::new()}
    }
    #[cfg(test)]
    fn from_vec(v: &Vec<MergeRange<T>>) -> Self {
        let mut range_set = RangeSet::new();
        for &range in v.iter() { range_set.push(range); }
        range_set
    }
    fn from_vec_with_overlap(v: &Vec<MergeRange<T>>) -> (Self, Self) {
        let mut range_set = RangeSet::new();
        let mut overlap_set = RangeSet::new();
        for &range in v.iter() {
            range_set.push_with_overlap(&mut overlap_set, range);
        }
        (range_set, overlap_set)
    }
    fn into_vec(self) -> Vec<MergeRange<T>> {
        self.ranges
    }
    fn push(&mut self, push_range: MergeRange<T>) {
        let mut overlap_set = RangeSet::new();
        self.push_with_overlap(&mut overlap_set, push_range);
    }
    fn push_with_overlap(&mut self, overlap_set: &mut Self,
                         push_range: MergeRange<T>) {
        let mut new_ranges = Vec::with_capacity(self.ranges.len() + 1);
        {
            // Drain the original range vector to create the new one.
            let mut range_iter = self.ranges.drain();
            let mut new_range = push_range;
            loop {
                match range_iter.next() {
                    Some(range) => match range.merge(new_range) {
                        // Nonoverlapping ranges. If this is the right place,
                        // insert the new range, otherwise move on.
                        Separate => if new_range.end < range.start {
                            new_ranges.push(new_range);
                            new_ranges.push(range);
                            new_ranges.extend(range_iter);
                            break;
                        } else {
                            new_ranges.push(range);
                        },
                        // If we can create a merged range, we still need to
                        // check and see if it can be merged with the next one
                        // before pushing it.
                        Adjacent(concat) => new_range = concat,
                        Overlap(union, overlap) => {
                            new_range = union;
                            overlap_set.push(overlap);
                        },
                    },
                    // If we reach here, the new range is last in the sequence.
                    None => {new_ranges.push(new_range); break;}
                }
            }
        }
        self.ranges = new_ranges;
    }
    fn complement(&self) -> Self {
        let mut complement_set = RangeSet::new();
        let len = self.ranges.len();
        // Treat an empty RangeSet specially.
        if len == 0 {
            complement_set.push(MergeRange::range_full());
            return complement_set;
        }
        // This is needed because a literal "1" can't be coerced to a "T".
        let one = <T as Int>::one();
        // Get the gap on the left boundary, if any.
        if self.ranges[0].start > (<T as Int>::min_value()) {
            complement_set.push(
                MergeRange::from_range_to(self.ranges[0].start - one)
                    );
        }
        // Get the gaps between ranges.
        for i in 1..len {
            complement_set.push(
                MergeRange::from_range(self.ranges[i-1].end + one,
                                       self.ranges[i].start - one)
                );
        }
        // Get the right boundary gap, if any.
        if self.ranges[len-1].end < (<T as Int>::max_value()) {
            complement_set.push(
                MergeRange::from_range_from(self.ranges[len-1].end + one)
                    );
        }
        complement_set
    }
}

#[cfg(test)]
mod range_set_tests {
    use super::RangeSet;
    use super::MergeRange;
    #[test]
    fn new_is_empty() {
        assert_eq!(RangeSet::<i16>::new().into_vec(), Vec::new());
    }
    #[test]
    fn single_contains_element() {
        let mut range_set = RangeSet::new();
        let range = MergeRange::from_range_to(1i16);
        range_set.push(range);
        assert_eq!(range_set.into_vec(), vec![range]);
    }
    #[test]
    fn separate_is_sorted() {
        let range1 = MergeRange::from_range(1u16, 5u16);
        let range2 = MergeRange::from_range_from(20u16);

        let mut range_set = RangeSet::new();
        range_set.push(range1);
        range_set.push(range2);
        assert_eq!(range_set.into_vec(), vec![range1, range2]);

        range_set = RangeSet::new();
        range_set.push(range2);
        range_set.push(range1);
        assert_eq!(range_set.into_vec(), vec![range1, range2]);
    }
    #[test]
    fn adjacent_is_combined() {
        let range1 = MergeRange::from_range(-2i8, 3);
        let range2 = MergeRange::from_range(4i8, 10);
        let merged = MergeRange::from_range(-2i8, 10);

        let mut range_set = RangeSet::new();
        range_set.push(range1);
        range_set.push(range2);
        assert_eq!(range_set.into_vec(), vec![merged]);

        range_set = RangeSet::new();
        range_set.push(range2);
        range_set.push(range1);
        assert_eq!(range_set.into_vec(), vec![merged]);
    }
    #[test]
    fn overlap_is_combined() {
        let range1 = MergeRange::from_range(4u32, 7);
        let range2 = MergeRange::from_range(6u32, 32);
        let merged = MergeRange::from_range(4u32, 32);

        let mut range_set = RangeSet::new();
        range_set.push(range1);
        range_set.push(range2);
        assert_eq!(range_set.into_vec(), vec![merged]);

        range_set = RangeSet::new();
        range_set.push(range2);
        range_set.push(range1);
        assert_eq!(range_set.into_vec(), vec![merged]);
    }
    #[test]
    fn from_vec_yields_ranges() {
        let range_vec = vec![
            MergeRange::from_range(6i64, 16),
            MergeRange::from_range_to(-10i64),
            MergeRange::from_range(33i64, 64),
            MergeRange::from_range(4i64, 7),
            ];
        let mut push_range_set = RangeSet::new();
        range_vec.iter().map(|x| push_range_set.push((*x).clone())).last();

        let vec_range_set = RangeSet::from_vec(&range_vec);
        assert_eq!(vec_range_set, push_range_set);
    }
    #[test]
    fn push_with_overlap_tracks_overlap() {
        let range_vec = vec![
            MergeRange::from_range(6i8, 16),
            MergeRange::from_range_to(-10i8),
            MergeRange::from_range_from(15i8),
            MergeRange::from_range(4i8, 7),
            ];
        let overlap_vec = vec![
            MergeRange::from_range(6i8, 7),
            MergeRange::from_range(15i8, 16),
            ];

        let mut range_set = RangeSet::new();
        let mut overlap_set = RangeSet::new();
        for &range in range_vec.iter() {
            range_set.push_with_overlap(&mut overlap_set, range);
        }
        assert_eq!(range_set, RangeSet::from_vec(&range_vec));
        assert_eq!(overlap_set, RangeSet::from_vec(&overlap_vec));
    }
    #[test]
    fn from_vec_with_overlap_tracks_overlap() {
        let range_vec = vec![
            MergeRange::from_range(6i8, 16),
            MergeRange::from_range_to(-10i8),
            MergeRange::from_range_from(15i8),
            MergeRange::from_range(4i8, 7),
            ];
        let overlap_vec = vec![
            MergeRange::from_range(6i8, 7),
            MergeRange::from_range(15i8, 16),
            ];

        let (range_set, overlap_set) =
            RangeSet::from_vec_with_overlap(&range_vec);
        assert_eq!(range_set, RangeSet::from_vec(&range_vec));
        assert_eq!(overlap_set, RangeSet::from_vec(&overlap_vec));
    }
    #[test]
    fn complement_yields_correct_set() {
        let range_vec = vec![
            MergeRange::from_range(10u32, 16),
            ];
        let complement_vec = vec![
            MergeRange::from_range_to(9u32),
            MergeRange::from_range_from(17u32),
            ];
        let range_set = RangeSet::from_vec(&range_vec);
        assert_eq!(range_set.complement(), RangeSet::from_vec(&complement_vec));
        assert_eq!(range_set.complement().complement(), range_set);
    }
    #[test]
    fn complement_range_full() {
        let range_full_vec = vec![MergeRange::<u64>::range_full()];
        let range_set = RangeSet::new();
        assert_eq!(range_set.complement(), RangeSet::from_vec(&range_full_vec));
        assert_eq!(range_set.complement().complement(), range_set);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MergeRange<T: Int> {
    start: T,
    end: T,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MergeResult<T: Int> {
    Separate,
    Adjacent(MergeRange<T>),
    Overlap(MergeRange<T>, MergeRange<T>),
}

impl<T: Int> MergeRange<T> {
    fn from_range(start: T, end: T) -> Self {
        debug_assert!(start <= end);
        MergeRange{start: start, end: end}
    }
    #[cfg(test)]
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
                Some(MergeRange::from_range(self.start, other.end))
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
            Overlap(MergeRange::from_range(min(self.start, other.start),
                                           max(self.end, other.end)),
                    MergeRange::from_range(max(self.start, other.start),
                                           min(self.end, other.end)))
        } else {
            Separate
        }
    }
}

#[cfg(test)]
mod merge_range_tests {
    use std::num::Int;
    use super::MergeRange;
    use super::MergeResult::*;
    #[test]
    fn unsigned_range_conversion() {
        assert_eq!(MergeRange::from_range(0u32, 20u32).to_range(),
                   (0u32, 20u32));
    }
    #[test]
    fn signed_range_conversion() {
        assert_eq!(MergeRange::from_range(-5i64, 5i64).to_range(),
                   (-5i64, 5i64));
        assert_eq!(MergeRange::from_range(0i32, 0i32).to_range(),
                   (0i32, 0i32));
    }
    #[test]
    fn range_to_conversion() {
        assert_eq!(MergeRange::from_range_to(2i8).to_range(),
                   (<i8 as Int>::min_value(), 2i8));
    }
    #[test]
    fn range_from_conversion() {
        assert_eq!(MergeRange::from_range_from(2u8).to_range(),
                   (2u8, <u8 as Int>::max_value()));
    }
    #[test]
    fn range_full_conversion() {
        assert_eq!(MergeRange::range_full().to_range(),
                   (<i32 as Int>::min_value(), <i32 as Int>::max_value()));
    }
    #[test]
    fn separate_ranges_not_merged() {
        let x = MergeRange::from_range(1i32, 2);
        let y = MergeRange::from_range(4i32, 5);
        assert_eq!(x.merge(y), Separate);
    }
    #[test]
    fn adjacent_ranges_concatenated() {
        let x = MergeRange::from_range(1i32, 2);
        let y = MergeRange::from_range(3i32, 5);
        assert_eq!(x.merge(y), Adjacent(MergeRange::from_range(1i32, 5)));
        assert_eq!(y.merge(x), x.merge(y));
    }
    #[test]
    fn adjacent_ranges_at_range_edge_concatenated() {
        let x = MergeRange::from_range_to(1u64);
        let y = MergeRange::from_range_from(2u64);
        assert_eq!(x.merge(y), Adjacent(MergeRange::range_full()));
        assert_eq!(y.merge(x), x.merge(y));
    }
    #[test]
    fn overlap_ranges_merged_with_overlap() {
        let x = MergeRange::from_range(-1i8, 2);
        let y = MergeRange::from_range(0i8, 5);
        assert_eq!(x.merge(y), Overlap(MergeRange::from_range(-1i8, 5),
                                       MergeRange::from_range(0i8, 2)));
        assert_eq!(y.merge(x), x.merge(y));
    }
    #[test]
    fn contained_range_merged_with_overlap() {
        let x = MergeRange::from_range(-1i8, 5);
        let y = MergeRange::from_range(0i8, 2);
        assert_eq!(x.merge(y), Overlap(MergeRange::from_range(-1i8, 5),
                                       MergeRange::from_range(0i8, 2)));
        assert_eq!(y.merge(x), x.merge(y));
    }
}
