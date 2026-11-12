//! What the generator doesn't generate, by the standard's sections, since every program passing
//! doesn't mean the language is covered (rule 3, D68). Each line comes off this list when the
//! generator learns it.

/// Generated today:
/// - values: `i32`, `i64`, `u8`, `u32` and `u64`; strings, as views from literals or other
///   views, and holding up to some bytes from literals that fit, counting a two-byte `é`, or from
///   a number with `as`, which fails when its text doesn't fit; bounded integers from literals that fit or
///   checked values; `f32.stable` and `f64.stable` from exact literals and ones marked `~`;
///   `const` from literals and other constants; `decl mut` with an `assign` on each path;
/// - maths: `+`, `-`, `x`, the `.wrap` and `.saturate` words, `div` and `mod` with `.trunc` and
///   `.floor`, a bare `div` on floats, the `.bit` operators, and `shl` and `shr` on unsigned
///   types; comparisons and comparison chains; `and`, `or`, `and.both`, `or.both` and `not`;
///   precedence with and without `( )`, as 3.9's table orders it;
/// - data: arrays from `[ ]` and `all`, and bounded lists from `[ ]` or `empty`, their items by
///   `pos:` and `offset.items:`, read and changed, and `append` to a bounded list; structs with integer fields in every layout, built with `[ ]`, their fields read
///   and changed; tagged unions whose cases carry integer data or none, and `match` covering every
///   case in any order, an arm naming its case to read its data;
/// - control: `let`, `change`, `if` and `else`, `while`, `for` over literal ranges and through
///   arrays, reading or changing each item, `forever`, `break`;
/// - functions: void ones taking integers lent to change with `Lend.mut`, and structs lent or
///   handed over with `Give` or `Copy`, called as statements; integer parameters and results, `fn.pure` ones calling only pure ones, `requires`
///   and `ensures result` contracts, labels on the last parameters, `labels.any_order`, and
///   defaults calls leave off;
/// - failures: functions that list failures, some carrying data, and `fail` them; calls handled
///   with `?`, `??`, `otherwise:` arms that use a value `instead`, or nothing, so a failure stops
///   the program;
/// - printing: `fmt_print` to an unbuffered writer, one to three values per print, with plain,
///   numbered or named holes, shapes for width, alignment, bases, `zeros`, `sign`, `places:` and
///   `form: science`, and escaped braces and `{tab}` in the text.
pub const NOT_GENERATED: &[&str] = &[
    "1.13 printing: `.stored`, named holes reading fields, widths in units other than `cols`, `unit`, other escapes, buffered writers, stderr",
    "2.3 `const` calling `fn.pure.baked` functions, float constants, constants whose maths overflows",
    "2.5 bounded parameters and fields, the test `if $n: T:` that checks a bound",
    "2.6, 2.7 `.fast` and `.ieee` floats, bounded floats, `exponent` literals, `fmt_print.stored`, floats in structs, arrays and calls",
    "2.9 heap strings, escapes in string literals, comparing strings, strings as values in prints, fields and calls",
    "2.10 and 2.12 growable lists and maps, allocators, `done`, `.len`, views and splits, a full list's failure handled",
    "2.14 `as` between other types, which isn't decided (12.3), `as` failures handled",
    "2.16 units, their conversions, points and spans",
    "2.18 `type.alias`, `type.new` and type families",
    "2.19 `decl` assigned in loops or `match` arms, or never changed",
    "2.21 `simd`, chunks and shuffles",
    "2.22 secrets",
    "3.9 chains that change direction, like `$a below $b above $c`",
    "3.10 a type's own operators",
    "4.2 the test, `if $n: T:`",
    "4.3 to 4.8 `match`, `for … in Give`, `for` over lists, a range's counter in use, ranges that step past their end, named loops",
    "5.x contracts that call pure functions, generics, interfaces, methods, `bake`, calling a function declared later, `Lent.mut` parameters",
    "6.x stored borrows, `must_Give`, `finish`, `defer`, giving a value away on one path, ownership words on functions that return values",
    "7.x structs holding other structs, lists or borrows, `must_Give` structs, generic structs, handing structs and unions to functions, the test `if $u: u.case:`",
    "8.x named sets of failures, a `case` reading a failure's data, arms that `return`, `break` or `fail`, handler narrowings and `*`",
    "9.x `use`, limbs, shields, `inline.asm`, pointers, `@concur`, `pub`",
    "10.x build modes beyond comparing outputs",
    "recursion, since calls only reach functions declared earlier",
];
