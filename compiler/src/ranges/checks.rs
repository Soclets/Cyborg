//! The decisions that drop a bounds or overflow check, proven with Verus (rule 13, D66, D312).
//!
//! The ranges pass works out the smallest and largest value each integer can have, then drops a check only when one
//! of these says it can't fail: every sum, difference or product of values in two ranges fits, every value in a range
//! is inside some bounds, or no value in a range is a given one, like 0 for a divisor. Each is proven for every value
//! in its ranges, so a dropped check is one that could never have fired.
//!
//! To rustc this is plain Rust with no dependency; the proofs only exist when Verus reads it, and
//! `tests/decision_66_proofs.rs` runs them.

#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

/// The smallest and largest value something can have, both included.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval {
    pub low: i128,
    pub high: i128,
}

#[cfg(verus_keep_ghost)]
verus! {

/// A value is in a range.
pub open spec fn contains(range: Interval, value: int) -> bool {
    range.low <= value && value <= range.high
}

/// Every value in a range is inside the bounds.
pub open spec fn all_inside(range: Interval, low: int, high: int) -> bool {
    forall|value: int| contains(range, value) ==> low <= value && value <= high
}

/// Every sum of a value in each range is inside the bounds.
pub open spec fn sums_inside(a: Interval, b: Interval, low: int, high: int) -> bool {
    forall|x: int, y: int| contains(a, x) && contains(b, y) ==> low <= x + y && x + y <= high
}

/// Every difference of a value in each range is inside the bounds.
pub open spec fn differences_inside(a: Interval, b: Interval, low: int, high: int) -> bool {
    forall|x: int, y: int| contains(a, x) && contains(b, y) ==> low <= x - y && x - y <= high
}

/// Every product of a value in each range is inside the bounds.
pub open spec fn products_inside(a: Interval, b: Interval, low: int, high: int) -> bool {
    forall|x: int, y: int| contains(a, x) && contains(b, y) ==> low <= x * y && x * y <= high
}

/// No value in a range is this one.
pub open spec fn never(range: Interval, value: int) -> bool {
    forall|x: int| contains(range, x) ==> x != value
}

/// Two ranges of values 0 and above, whose largest product is inside the bounds.
pub open spec fn corners_inside(a: Interval, b: Interval, low: int, high: int) -> bool {
    0 <= a.low && 0 <= b.low && low <= 0 && (a.high as int) * (b.high as int) <= high
}

/// For values 0 and above, the largest product is the product of the largest values, so corners bound them all.
pub proof fn corners_bound_every_product(a: Interval, b: Interval, low: int, high: int)
    requires corners_inside(a, b, low, high),
    ensures products_inside(a, b, low, high),
{
    assert forall|x: int, y: int| contains(a, x) && contains(b, y) implies low <= x * y && x * y <= high by {
        assert(0 <= x * y) by (nonlinear_arith) requires 0 <= x, 0 <= y;
        assert(x * y <= (a.high as int) * (b.high as int)) by (nonlinear_arith)
            requires 0 <= x, x <= a.high, 0 <= y, y <= b.high;
    }
}

}

// trace: D312
/// Whether every value in a range is inside bounds, so a bound check on it can't fail.
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result ==> all_inside(range, low as int, high as int)))]
pub(crate) fn inside(range: Interval, low: i128, high: i128) -> bool {
    low <= range.low && range.high <= high
}

// trace: D312
/// Whether every sum of values in two ranges is inside bounds, so an overflow check on it can't fail.
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result ==> sums_inside(a, b, low as int, high as int)))]
pub(crate) fn sum_inside(a: Interval, b: Interval, low: i128, high: i128) -> bool {
    match (a.low.checked_add(b.low), a.high.checked_add(b.high)) {
        (Some(smallest), Some(largest)) => low <= smallest && largest <= high,
        _ => false,
    }
}

// trace: D312
/// Whether every difference of values in two ranges is inside bounds, so an overflow check on it can't fail.
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result ==> differences_inside(a, b, low as int, high as int)))]
pub(crate) fn difference_inside(a: Interval, b: Interval, low: i128, high: i128) -> bool {
    match (a.low.checked_sub(b.high), a.high.checked_sub(b.low)) {
        (Some(smallest), Some(largest)) => low <= smallest && largest <= high,
        _ => false,
    }
}

// trace: D312
/// Whether every product of values in two ranges of values 0 and above is inside bounds. Ranges reaching below 0
/// aren't worked out, so their checks stay.
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result ==> corners_inside(a, b, low as int, high as int)))]
pub(crate) fn product_inside(a: Interval, b: Interval, low: i128, high: i128) -> bool {
    if a.low < 0 || b.low < 0 || low > 0 {
        return false;
    }
    match a.high.checked_mul(b.high) {
        Some(largest) => largest <= high,
        None => false,
    }
}

// trace: D312
/// Whether no value in a range is this one, as a divisor that's never 0.
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result ==> never(range, value as int)))]
pub(crate) fn excludes(range: Interval, value: i128) -> bool {
    value < range.low || range.high < value
}

// trace: D312
/// The values in `a` that can be below some value in `b`, as when a loop runs while `a below b`. Empty when none can.
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    ensures forall|x: int, y: int| contains(a, x) && contains(b, y) && x < y ==> contains(result, x)))]
pub(crate) fn below(a: Interval, b: Interval) -> Interval {
    match b.high.checked_sub(1) {
        Some(most) if most < a.high => Interval { low: a.low, high: most },
        _ => a,
    }
}

// trace: D312
/// The values in `a` that can be at most some value in `b`.
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    ensures forall|x: int, y: int| contains(a, x) && contains(b, y) && x <= y ==> contains(result, x)))]
pub(crate) fn at_most(a: Interval, b: Interval) -> Interval {
    if b.high < a.high { Interval { low: a.low, high: b.high } } else { a }
}

// trace: D312
/// The values in `a` that can be above some value in `b`.
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    ensures forall|x: int, y: int| contains(a, x) && contains(b, y) && x > y ==> contains(result, x)))]
pub(crate) fn above(a: Interval, b: Interval) -> Interval {
    match b.low.checked_add(1) {
        Some(least) if least > a.low => Interval { low: least, high: a.high },
        _ => a,
    }
}

// trace: D312
/// The values in `a` that can be at least some value in `b`.
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    ensures forall|x: int, y: int| contains(a, x) && contains(b, y) && x >= y ==> contains(result, x)))]
pub(crate) fn at_least(a: Interval, b: Interval) -> Interval {
    if b.low > a.low { Interval { low: b.low, high: a.high } } else { a }
}

// trace: D312
/// Every value in either range, for where two paths meet.
#[cfg_attr(verus_keep_ghost, verus_spec(result =>
    ensures forall|x: int| contains(a, x) || contains(b, x) ==> contains(result, x)))]
pub(crate) fn join(a: Interval, b: Interval) -> Interval {
    let low = if a.low < b.low { a.low } else { b.low };
    let high = if a.high > b.high { a.high } else { b.high };
    Interval { low, high }
}
