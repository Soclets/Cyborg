//! What the generator doesn't generate, by the standard's sections, since every program passing
//! doesn't mean the language is covered (rule 3, D68). Each line comes off this list when the
//! generator learns it.

/// Generated today:
/// - values: `i32`, `i64`, `u8`, `u32` and `u64`; strings, as views from literals or other
///   views, and holding up to some bytes from literals that fit, counting a two-byte `é`, or from
///   a number with `as`, which fails when its text doesn't fit; bounded integers from literals that fit or
///   checked values; `f32.stable` and `f64.stable` from exact literals and ones marked `~`;
///   `const` from literals and other constants; `type.alias` names used as their integer, and
///   `type.new` names, alone or in a `fam.type.new` family, crossing in and out with `as`; `decl mut` with an `assign` on each path;
/// - maths: `+`, `-`, `x`, the `.wrap` and `.saturate` words, `div` and `mod` with `.trunc` and
///   `.floor`, a bare `div` on floats, the `.bit` operators, and `shl` and `shr` on unsigned
///   types; comparisons and comparison chains; `and`, `or`, `and.both`, `or.both` and `not`;
///   precedence with and without `( )`, as 3.9's table orders it;
/// - secrets: integer `.secret` values from literals or plain values, joined by `.bit`, `.wrap`
///   and `.saturate` operators, chosen between with `.%pick` on secrets compared, and made public
///   with `.%reveal()` in a shield;
/// - data: `simd` values from `[ ]` and `all`, maths lane by lane on two of one type, their
///   `sum`, `product`, `min_val` and `max_val`, and lane comparisons read with `any_true` and
///   `all_true`; arrays from `[ ]` and `all`, and bounded lists from `[ ]` or `empty`, their items by
///   `pos:` and `offset.items:`, read and changed, and `append`; in `main`, lists that grow on the
///   heap or in an arena with a random `room:`, from `list.from` or `empty`, ended by `defer`; structs with integer fields in every layout, built with `[ ]`, their fields read
///   and changed, and methods on them that read their fields or change one, called with `.%` and
///   `.%%`; tagged unions whose cases carry integer data or none, and `match` covering every
///   case in any order, an arm naming its case to read its data;
/// - control: `let`, `change`, `if` and `else`, the test `if $v: T:` on a union's case or an
///   integer's bound, `while`, `for` over literal ranges and through
///   arrays, reading or changing each item, `forever`, `break`;
/// - functions: printed in a shuffled order, so calls reach ones declared later; calls started
///   as `@concur` tasks on `std::run::one_thread` and ended with `task.done`; void ones taking integers lent to change with `Lend.mut`, and structs lent or
///   handed over with `Give` or `Copy`, called as statements; integer parameters and results, `fn.pure` ones calling only pure ones, `requires`
///   and `ensures result` contracts, labels on the last parameters, `labels.any_order`, and
///   defaults calls leave off;
/// - failures: functions that list failures, some carrying data, and `fail` them; calls handled
///   with `?`, `??`, `otherwise:` arms that use a value `instead`, or nothing, so a failure stops
///   the program;
/// - units: `fam.unit` families of whole multiples, integer values in them, added, bounded in
///   units, converted with `as` and its narrowings, taken off with `as`; float units with two
///   bases, a `fam.prefix` the length opts into `with`, units made with `per`, `per` of a `per`
///   and `x`, built by `div` and `x` where they're stored; an offset family and a `decibels of`
///   one, whose points and spans add, subtract and convert; `std`'s angles converted to radians
///   for `std::math::sin`; parameters whose label carries a unit, given a plain number;
/// - `must_Give`: a `struct.must_Give` with integer fields, ended by a function that `finish`es
///   it, made by one that returns it and handed through one that takes it `Given` and returns
///   it, and a `must_Give` holder whose field is moved out before it's `finish`ed; each value is
///   ended straight away, by `defer`, in both arms of an `if`, after being passed through, or
///   inside its holder, and a field may be read on the way;
/// - maps: `std::coll::map.hash` and `std::coll::map.ordered` with `key.type:` and `val.type:`,
///   made `empty` in `main` and ended by a `defer` of `std::coll::map.done`;
/// - generics: the interfaces `equatable`, `ordered`, which `includes` it and states a `law`,
///   and `addable`; the `type_set` `whole`; `satisfies` lines, one now and then written `.|.`;
///   a struct with its own `below`, `==` and `+`; `gmax`, `gsame`, `gsum`, `gtwice` and
///   `gkeep` called with numbers bare and the struct lent, inside maths and inside each other;
///   `gbox<?item: any.type>` holding an integer; and `glarger` with a unit parameter;
/// - shields: around `.%reveal()`; around `ptr.raw`, `ptr.raw.mut`, `ptr.volatile` and
///   `ptr.volatile.mut` pointers to integer variables made with `.of` and `Lend` or
///   `Lend.mut`, read with `.%read()`, written with `.%%write(val:)`, and moved on in items and
///   back in bytes with `.%moved`; and around one aarch64 `inline.asm` instruction in an
///   `on.target aarch64:` branch whose `else:` works out the same value; and a
///   `union.@csup::untagged` declared in a shield, built in another with one integer field and
///   read back as a field no wider;
/// - printing: `fmt_print` to an unbuffered writer, one to three values per print, with plain,
///   numbered or named holes, shapes for width, alignment, bases, `zeros`, `sign`, `places:` and
///   `form: science`, and escaped braces and `{tab}` in the text.
pub const NOT_GENERATED: &[&str] = &[
    "1.13 printing: `.stored`, named holes reading fields, widths in units other than `cols`, other escapes, buffered writers, stderr",
    "2.3 `const` calling `fn.pure.baked` functions, float constants, constants whose maths overflows",
    "2.5 bounded parameters and fields",
    "2.6, 2.7 `.fast` and `.ieee` floats, bounded floats, `exponent` literals, `fmt_print.stored`, floats in structs, arrays and calls",
    "2.9 heap strings, escapes in string literals, comparing strings, strings as values in prints, fields and calls",
    "2.10 `.len`, views and splits, a full list's failure handled, growable lists in functions, `grow:`",
    "2.12 undecided: putting, getting and going through a map's items",
    "2.14 `as` between other types, which isn't decided (12.3), `as` failures handled",
    "2.16 still to do: `use std::units::… as …`, `std`'s `time.seconds` as integer points and spans, and units on generic parameters, which come with generics; undecided: `std`'s other units (12.2)",
    "2.18 named types over structs, lists or strings, `fam.type.alias`, maths on a `type.new`",
    "2.19 `decl` assigned in loops or `match` arms, or never changed",
    "2.21 `simd` with one value, chunks, shuffles, `pick`, lookups, `lanes: max`, floats in lanes, `as` from arrays",
    "2.22 `bool.secret` values with `and.both`, `or.both` and `not`, secrets in arrays and `simd`, shifts on secrets",
    "3.9 chains that change direction, like `$a below $b above $c`",
    "3.10 a type's own operators other than `below`, `==` and `+`, like `x` and a `div` narrowing",
    "4.3 to 4.8 `match`, `for … in Give`, `for` over lists, a range's counter in use, ranges that step past their end, named loops",
    "5.x contracts that call pure functions, methods with labels or on other types, `bake`; generics with more than one type parameter, and generic structs holding structs; undecided: specialization, value parameters, type slots and how laws are proven (12.5)",
    "6.x stored borrows and `.held`, `Lend from` results, `must_Give` values that leave a loop or a `match`, and `defer` on anything but ending a value",
    "7.x structs holding other structs, lists or borrows, beyond the `must_Give` holder, handing unions to functions",
    "8.x named sets of failures, a `case` reading a failure's data, arms that `return`, `break` or `fail`, handler narrowings and `*`",
    "9.x still to do: tasks on `os::threads` and `os::events`, tasks that borrow, locks and atomics; `pub` and `use` between packages, once programs have more than one; NotBuiltYet: `.at(address:)`, which points at hardware the oracle doesn't have; undecided: a shield's own `requires` and `ensures` (12.9), your own allocator (2.10), and reading more of a C union than its written field filled (asked)",
    "10.x build modes beyond comparing outputs",
    "recursion, since calls only reach functions declared earlier",
];
