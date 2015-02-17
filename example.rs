
extern crate int_range_check;

use std::fmt::Display;
use std::num::Int;

use int_range_check::uncovered_and_overlapped;
use int_range_check::IntRange;
use int_range_check::IntRange::*;

fn main() {
    example_driver("Example 1", vec![Bound(0i32, 5i32), From(3)]);

    example_driver("Example 2a", vec![To(5u8), From(250)]);
    example_driver("Example 2b", vec![Bound(0u8, 5), Bound(250, 255)]);
}

fn example_driver<T: Display+Int>(title: &str, ranges: Vec<IntRange<T>>) {
    let (uncovered, overlapped) =
        uncovered_and_overlapped(&ranges);
    println!("{} input ranges: {}", title, ranges);
    println!("{} uncovered ranges: {}", title, uncovered);
    println!("{} overlapping ranges: {}", title, overlapped);
}
