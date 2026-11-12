//! What the generator doesn't generate, by the standard's sections, since every program passing
//! doesn't mean the language is covered (rule 3, D68). Each line comes off this list when the
//! generator learns it.

/// Generated today: `i32`, `i64`, `u8`, `u32` and `u64` values and variables; `+`, `-`, `x`, the
/// `.wrap` and `.saturate` words, `div` and `mod` with `.trunc` and `.floor`, the `.bit`
/// operators, and `shl` and `shr` on unsigned types; comparisons; `and`, `or` and `not`;
/// functions that list failures, some carrying data, and `fail` them; calls handled with `?`,
/// `??`, `otherwise:` arms that use a value `instead`, or nothing, so a failure stops the
/// program; arrays from `[ ]` and `all`, their items by `pos:` and `offset.items:`, read and changed;
/// precedence with and without `( )`, as 3.9's table orders it; comparison chains; `let`,
/// `change`, `if` and `else`, `while`, `for` over literal ranges, `break`; functions with
/// integer parameters and results; printing a variable with `fmt_print` to an unbuffered writer.
pub const NOT_GENERATED: &[&str] = &[
    "1.13 printing: shapes, named holes, `.stored`, more than one value, buffered writers, stderr",
    "2.3 `const`",
    "2.5 bounded integers and the checks they bring",
    "2.6, 2.7 floats, `.stable`, `.fast`, `.ieee` and inexact literals",
    "2.9 strings: views, bounded and heap strings",
    "2.10 and 2.12 lists and maps, `append`, allocators, `done`, views and splits",
    "2.14 `as`, and its failures",
    "2.16 units, their conversions, points and spans",
    "2.18 `type.alias`, `type.new` and type families",
    "2.19 `decl` and `assign`",
    "2.21 `simd`, chunks and shuffles",
    "2.22 secrets",
    "3.6 `and.both` and `or.both`",
    "3.9 chains that change direction, like `$a below $b above $c`",
    "3.10 a type's own operators",
    "4.2 the test, `if $n: T:`",
    "4.3 to 4.8 `match`, `for` over collections, a range's counter in use, ranges that step past their end, `forever`, named loops",
    "5.x labels, defaults, `fn.pure`, contracts, generics, interfaces, methods, `bake`",
    "6.x ownership words, `Lend.mut` parameters, stored borrows, `must_Give`, `finish`, `defer`",
    "7.x structs, unions and `match` on them",
    "8.x named sets of failures, a `case` reading a failure's data, arms that `return`, `break` or `fail`, handler narrowings and `*`",
    "9.x `use`, limbs, shields, `inline.asm`, pointers, `@concur`, `pub`",
    "10.x build modes beyond comparing outputs",
    "recursion, since calls only reach functions declared earlier",
];
