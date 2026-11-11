//! The decisions at the heart of the ownership checks, proven with Verus (rule 13, D66, D67).
//!
//! The flow pass follows every path through a function and joins what each path knows where they meet. Whether a use
//! is refused, whether a duty is still open, and whether a borrow is spoiled all come down to the few functions here.
//! What they promise is what makes the pass sound: a value given away on any path is never usable after the paths
//! meet, a duty left open on any path is always reported, and a borrow spoiled on any path stays spoiled.
//!
//! To rustc this is plain Rust with no dependency; the proofs only exist when Verus reads it, and
//! `tests/decision_66_proofs.rs` runs them. The spans and messages around these decisions are plumbing, and the oracle
//! comparison covers them (D67).

// Verus reads this file, and knows plain matches and loops better than these library calls.
#![allow(clippy::match_like_matches_macro, clippy::needless_range_loop)]

#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

// trace: D2, D185
/// What the paths reaching a point know about one variable, reduced to what the checks decide on.
#[cfg_attr(verus_keep_ghost, verus_verify)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Held whole on every path.
    Alive,
    /// A field was moved out on some path, and nothing was given away whole (D185).
    Partial,
    /// Given away or finished on some paths, but not all.
    MaybeGone,
    /// Given away or finished on every path.
    Gone,
}

#[cfg(verus_keep_ghost)]
verus! {

/// Given away or finished on at least one path.
pub open spec fn gone_somewhere(kind: Kind) -> bool {
    kind == Kind::MaybeGone || kind == Kind::Gone
}

/// Given away or finished on every path.
pub open spec fn gone_everywhere(kind: Kind) -> bool {
    kind == Kind::Gone
}

/// Touched on at least one path: a field moved out, or the whole given away.
pub open spec fn touched_somewhere(kind: Kind) -> bool {
    kind != Kind::Alive
}

/// Where two sets of paths meet.
pub open spec fn joined(a: Kind, b: Kind) -> Kind {
    if a == Kind::Gone && b == Kind::Gone {
        Kind::Gone
    } else if gone_somewhere(a) || gone_somewhere(b) {
        Kind::MaybeGone
    } else if a == Kind::Partial || b == Kind::Partial {
        Kind::Partial
    } else {
        Kind::Alive
    }
}

/// The order the pass climbs as it goes round loops: a join never forgets that something happened.
pub open spec fn below(a: Kind, b: Kind) -> bool {
    joined(a, b) == b
}

/// Joining in any order gives the same answer, so the order paths are visited in doesn't matter.
pub proof fn join_commutes(a: Kind, b: Kind)
    ensures joined(a, b) == joined(b, a),
{
}

pub proof fn join_associates(a: Kind, b: Kind, c: Kind)
    ensures joined(joined(a, b), c) == joined(a, joined(b, c)),
{
}

pub proof fn join_is_idempotent(a: Kind)
    ensures joined(a, a) == a,
{
}

/// A join is above both sides, so going round a loop again can only climb, and the lattice has four steps, so the pass
/// ends.
pub proof fn join_is_above_both(a: Kind, b: Kind)
    ensures below(a, joined(a, b)), below(b, joined(a, b)),
{
}

/// A path that gave the value away is never lost at a join.
pub proof fn join_keeps_every_give(a: Kind, b: Kind)
    ensures
        gone_somewhere(joined(a, b)) == (gone_somewhere(a) || gone_somewhere(b)),
        gone_everywhere(joined(a, b)) == (gone_everywhere(a) && gone_everywhere(b)),
        touched_somewhere(joined(a, b)) == (touched_somewhere(a) || touched_somewhere(b)),
{
}

/// Whether a path through fields starts with another.
pub open spec fn starts_with(long: Seq<usize>, short: Seq<usize>) -> bool {
    short.len() <= long.len() && long.subrange(0, short.len() as int) == short
}

pub open spec fn shorter_len(a: Seq<usize>, b: Seq<usize>) -> int {
    if a.len() < b.len() { a.len() as int } else { b.len() as int }
}

/// Two paths agree as far as the shorter goes.
pub open spec fn agree(a: Seq<usize>, b: Seq<usize>) -> bool {
    forall|each: int| 0 <= each < shorter_len(a, b) ==> a[each] == b[each]
}

/// Agreeing is exactly one path being the start of the other, so `overlaps` finds every field that's the same place,
/// or inside it.
pub proof fn agree_means_one_starts_the_other(a: Seq<usize>, b: Seq<usize>)
    ensures agree(a, b) == (starts_with(a, b) || starts_with(b, a)),
{
    if a.len() <= b.len() {
        if agree(a, b) {
            assert(b.subrange(0, a.len() as int) =~= a);
        }
        if starts_with(b, a) {
            assert forall|each: int| 0 <= each < shorter_len(a, b) implies a[each] == b[each] by {
                assert(b.subrange(0, a.len() as int)[each] == b[each]);
            }
        }
        if starts_with(a, b) {
            assert forall|each: int| 0 <= each < shorter_len(a, b) implies a[each] == b[each] by {
                assert(a.subrange(0, b.len() as int)[each] == a[each]);
            }
        }
    } else {
        if agree(a, b) {
            assert(a.subrange(0, b.len() as int) =~= b);
        }
        if starts_with(a, b) {
            assert forall|each: int| 0 <= each < shorter_len(a, b) implies a[each] == b[each] by {
                assert(a.subrange(0, b.len() as int)[each] == a[each]);
            }
        }
    }
}

}

// trace: D2, D185, D285
/// Where two sets of paths meet.
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result == joined(a, b)))]
pub(crate) fn join(a: Kind, b: Kind) -> Kind {
    match (a, b) {
        (Kind::Gone, Kind::Gone) => Kind::Gone,
        (Kind::Gone | Kind::MaybeGone, _) | (_, Kind::Gone | Kind::MaybeGone) => Kind::MaybeGone,
        (Kind::Partial, _) | (_, Kind::Partial) => Kind::Partial,
        (Kind::Alive, Kind::Alive) => Kind::Alive,
    }
}

// trace: D2
/// Whether the whole value can be used: read, lent, given or changed. Not after it was given away or finished on any
/// path, nor once a field was moved out (D185).
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result == !touched_somewhere(kind)))]
pub(crate) fn whole_usable(kind: Kind) -> bool {
    match kind {
        Kind::Alive => true,
        Kind::Partial | Kind::MaybeGone | Kind::Gone => false,
    }
}

// trace: D2, D129
/// Whether a `must_Give` duty is still open when leaving: unless it was given away or finished on every path.
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result == !gone_everywhere(kind)))]
pub(crate) fn duty_open(kind: Kind) -> bool {
    match kind {
        Kind::Gone => false,
        Kind::Alive | Kind::Partial | Kind::MaybeGone => true,
    }
}

// trace: D2, D169
/// Whether a use of a borrowed value spoils a stored borrow of it: any use spoils one lent to change, and anything but
/// a read spoils one lent to read.
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result == (lent_to_change || !reading)))]
pub(crate) fn spoils(lent_to_change: bool, reading: bool) -> bool {
    lent_to_change || !reading
}

// trace: D169
/// A borrow spoiled on any path stays spoiled where the paths meet.
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result == (a || b)))]
pub(crate) fn join_spoiled(a: bool, b: bool) -> bool {
    a || b
}

// trace: D173
/// Whether two places overlap, as fields of the same variable: they agree as far as the shorter goes, so one is the
/// start of the other (`agree_means_one_starts_the_other`). Different fields are separate, and a field overlaps the whole
/// (D173).
#[cfg_attr(verus_keep_ghost, verus_spec(result => ensures result == agree(a@, b@)))]
pub(crate) fn overlaps(a: &[usize], b: &[usize]) -> bool {
    let shorter = if a.len() < b.len() { a.len() } else { b.len() };
    let mut index = 0;
    #[cfg_attr(verus_keep_ghost, verus_spec(
        invariant
            shorter == shorter_len(a@, b@),
            index <= shorter,
            forall|each: int| 0 <= each < index ==> a@[each] == b@[each],
        decreases shorter - index,
    ))]
    while index < shorter {
        if a[index] != b[index] {
            return false;
        }
        index += 1;
    }
    true
}
