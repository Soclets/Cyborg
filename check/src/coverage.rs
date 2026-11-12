//! What the generator doesn't generate, by the standard's sections, since every program passing
//! doesn't mean the language is covered (rule 3, D68). Each line comes off this list when the
//! generator learns it.

/// Generated today:
/// - values: `i32`, `i64`, `u8`, `u32` and `u64`; strings, as views from literals or other
///   views, and holding up to some bytes from literals that fit, counting a two-byte `é`, or from
///   a number with `as`, which fails when its text doesn't fit; bounded integers from literals that fit or
///   checked values, as variables, parameters and struct fields; `f32.stable` and `f64.stable` from exact literals, ones marked `~`, and
///   `exponent` ones; `.ieee` floats, dividing by zero now and then, crossing into `.stable`
///   with `as`; `.fast` floats; bounded floats;
///   `const` from literals, other constants and calls to `fn.pure.baked` functions, and float
///   `const`s; `bake` on such a call, and the same functions called while running; `type.alias` names used as their integer, and
///   `type.new` names, alone or in a `fam.type.new` family, crossing in and out with `as`; `decl` and `decl mut` with an `assign` straight after, in both arms of an `if`, or in every arm of a `match`;
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
///   heap or in an arena with a random `room:`, from `list.from` or `empty`, ended by `defer`,
///   in functions with a heap of their own too, some saying `grow:`; appends handled with
///   `?? void`; lists read through views, `std::coll::list<…>` and `.mut`, split into halves,
///   and their `.len` taken off its unit; structs with integer fields in every layout, built with `[ ]`, their fields read
///   and changed, and methods on them that read their fields or change one, called with `.%` and
///   `.%%`; tagged unions whose cases carry integer data or none, and `match` covering every
///   case in any order, an arm naming its case to read its data;
/// - loops: `for` over arrays and lists with `Lend` or `Lend.mut`, over an array of `must_Give`
///   values with `Give`, and over ranges whose counter is read, whose end lies past the last
///   step or is only known while running; named loops, stopped with `break name` from inside;
/// - control: `let`, `change`, `if` and `else`, the test `if $v: T:` on a union's case or an
///   integer's bound, `while`, `for` over literal ranges and through
///   arrays, reading or changing each item, `forever`, `break`;
/// - functions: printed in a shuffled order, so calls reach ones declared later; calls started
///   as `@concur` tasks on `std::run::one_thread` and ended with `task.done`; void ones taking integers lent to change with `Lend.mut`, and structs lent or
///   handed over with `Give` or `Copy`, called as statements; integer parameters and results, `fn.pure` ones calling only pure ones, `requires`
///   and `ensures result` contracts, labels on the last parameters, `labels.any_order`, and
///   defaults calls leave off;
/// - failures: functions that list failures, some carrying data, or a `fam.failure` set whole,
///   and `fail` them, a set's by their full names; calls handled with `?`, `??`, `otherwise:`
///   arms that use a value `instead`, the failure's own data through a named `case`, or
///   `return`, `break` or `fail`, a set's `fam.handler` narrowing, or one that returns marked
///   `*`, or nothing, so a failure stops the program;
/// - units: `fam.unit` families of whole multiples, integer values in them, added, bounded in
///   units, converted with `as` and its narrowings, taken off with `as`; float units with two
///   bases, a `fam.prefix` the length opts into `with`, units made with `per`, `per` of a `per`
///   and `x`, built by `div` and `x` where they're stored; an offset family and a `decibels of`
///   one, whose points and spans add, subtract and convert; `std`'s angles converted to radians
///   for `std::math::sin`, now and then shortened with `use`; `std`'s `time.seconds` as `i64`
///   points and spans; parameters whose label carries a unit, given a plain number;
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
///   `rest = all 0`, and read back as any field;
/// - recursion: `rsum`, which calls itself, and `reven` and `rodd`, which call each other;
/// - printing: `fmt_print` and `fmt_print.stored` to an unbuffered writer, to stderr, or to a
///   buffered one that's flushed now and then and finished with `os::done`; one to three values
///   per print, with plain, numbered or named holes, named ones reading struct fields too;
///   shapes for width in every unit but points, alignment, bases, `zeros`, `sign`, `places:` and
///   `form: science`; and escapes, braces, `{tab}` and `{newln}` in the text; heap strings made
///   by `str.from` from text or a number, ended by a deferred `str.done`.
pub const NOT_GENERATED: &[&str] = &[
    "1.13 undecided: what a point is, for a width (12.2); `places:` or `form:` with `.stored`; a width with no `align:`",
    "2.6, 2.7 still to do: floats in structs, arrays and calls; undecided: a negative exponent (12.2)",
    "2.9 still to do: strings in struct fields and function calls; undecided: escapes in string literals and how literals work (12.2), and `==` on `str` (12)",
    "2.10 undecided: a full list's failure named in an `otherwise` arm (12.7)",
    "2.12 undecided: putting, getting and going through a map's items",
    "2.14 `as` between other types, which isn't decided (12.3), `as` failures handled",
    "2.16 undecided: `std`'s units other than `time.seconds` and `angle`'s (12.2)",
    "2.18 named types over structs, lists or strings, `fam.type.alias`, maths on a `type.new`",
    "2.21 `simd` with one value, chunks, shuffles, `pick`, lookups, `lanes: max`, floats in lanes, `as` from arrays",
    "2.22 `bool.secret` values with `and.both`, `or.both` and `not`, secrets in arrays and `simd`, shifts on secrets",
    "4.4 `for` over a fresh collection, since generated functions don't return arrays or lists yet",
    "5.x contracts that call pure functions, methods with labels or on other types; generics with more than one type parameter, and generic structs holding structs; undecided: specialization, value parameters, type slots and how laws are proven (12.5)",
    "6.x stored borrows and `.held`, `Lend from` results, `must_Give` values that leave a loop or a `match`, and `defer` on anything but ending a value",
    "7.x structs holding other structs, lists or borrows, beyond the `must_Give` holder, handing unions to functions",
    "9.x still to do: tasks on `os::threads` and `os::events`, tasks that borrow, locks and atomics; `pub` and `use` between packages, once programs have more than one; NotBuiltYet: `.at(address:)`, which points at hardware the oracle doesn't have; undecided: a shield's own `requires` and `ensures` (12.9), your own allocator (2.10)",
    "10.x compiler-only: build modes, since the oracle runs one way and the live comparison runs every mode against it",
    "recursion in generated functions, beyond the three fixed ones",
];
