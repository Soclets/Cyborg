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
///   `type.new` names, alone or in a `fam.type.new` or `fam.type.alias` family, crossing in and
///   out with `as`; a `type.new` over a struct, crossed into with `Copy` or `Give` and out with
///   `Give`; aliases for a bounded list and a bounded string; `decl` and `decl mut` with an `assign` straight after, in both arms of an `if`, or in every arm of a `match`;
/// - maths: `+`, `-`, `x`, the `.wrap` and `.saturate` words, `div` and `mod` with `.trunc` and
///   `.floor`, a bare `div` on floats, the `.bit` operators, and `shl` and `shr` on unsigned
///   types; comparisons and comparison chains; `and`, `or`, `and.both`, `or.both` and `not`;
///   precedence with and without `( )`, as 3.9's table orders it;
/// - secrets: integer `.secret` values from literals or plain values, joined by `.bit`, `.wrap`
///   and `.saturate` operators, chosen between with `.%pick` on secrets compared, and made public
///   with `.%reveal()` in a shield; shifted right by a public amount; `bool.secret` from
///   comparing secrets, joined with `and.both`, `or.both` and `not`, choosing with `.%pick`;
///   secrets in an array, one read back;
/// - data: `simd` values from `[ ]` and `all`, maths lane by lane on two of one type, their
///   `sum`, `product`, `min_val` and `max_val`, and lane comparisons read with `any_true` and
///   `all_true`; arrays from `[ ]` and `all`, and bounded lists from `[ ]` or `empty`, their items by
///   `pos:` and `offset.items:`, read and changed, and `append`; `simd` from arrays with `as`
///   and back, chosen lane by lane with `.%pick`, shuffled with `pos:` or `offset.lanes:`, and
///   loops over `simd_chunks` that pad, need an exact multiple, or hand over leftovers, and
///   `.%%simd_chunks.mut` ones that change each chunk; `.%reverse`, `.%rotate.left` and
///   `.right`, `.%join`, `.%interleave`, and `.%lookup`; `not.bit` on integers and secrets; in `main`, lists that grow on the
///   heap or in an arena with a random `room:`, from `list.from` or `empty`, ended by `defer`,
///   in functions with a heap of their own too, some saying `grow:`; appends handled with
///   `?? void`; lists read through views, `std::coll::list<…>` and `.mut`, split into halves,
///   and their `.len` taken off its unit; structs with integer fields in every layout, built with `[ ]`, their fields read
///   and changed, and methods on them that read their fields or change one, called with `.%` and
///   `.%%`; tagged unions whose cases carry integer data or none, and `match` covering every
///   case in any order, an arm naming its case to read its data;
/// - loops: `for` over a fresh array a call returns, bare; over arrays and lists with `Lend` or
///   `Lend.mut`, over an array of `must_Give`
///   values with `Give`, and over ranges whose counter is read, whose end lies past the last
///   step or is only known while running; named loops, stopped with `break name` from inside;
/// - control: `let`, `change`, `if` and `else`, the test `if $v: T:` on a union's case or an
///   integer's bound, `while`, `for` over literal ranges and through
///   arrays, reading or changing each item, `forever`, `break`;
/// - methods: `%%` ones whose `$by` has a label, and `.%size` on a union, which matches it
///   and returns from every arm; contracts calling pure functions;
/// - functions: printed in a shuffled order, so calls reach ones declared later; calls started
///   as `@concur` tasks on `std::run::one_thread`, `os::threads` or `os::events`, with
///   `alloc:` and `stack:` where the runner switches tasks, and ended with `task.done`; tasks
///   reading split halves of a list; atomics added to, swapped, compared and swapped with the
///   lost race handled, and read, now and then `.acquire`; one or two locks taken in one
///   `@concur::locked` block; void ones taking integers lent to change with `Lend.mut`, and structs lent or
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
/// - structs holding structs: `nest`, holding a struct, an `i32` and a bounded list, built
///   whole, a field of its struct changed and an item of its list read; `view`, holding a borrow
///   of a struct, read through; a union lent to a function;
/// - more structs: `fpoint`, with float fields, one changed through `fhalf`, a function on a
///   float; float arrays and a float `simd` summed; a union given to a function; `bag`, a
///   `must_Give` struct holding a growable list, ended by giving the list field to
///   `list.done` and then `finish`;
/// - borrows: stored borrows of a struct, read through, `.held` in a block of their own, and
///   `Lend.mut` written through; a function returning a borrow of one of two structs, stored
///   as `Lend from <$a, $b>`; `defer` on a print or a `change`; a `must_Give` value ended in
///   every arm of a `match`;
/// - maps: `std::coll::map.hash` and `std::coll::map.ordered` with `key.type:` and `val.type:`,
///   made `empty` in `main` and ended by a `defer` of `std::coll::map.done`;
/// - generics: the interfaces `equatable`, `ordered`, which `includes` it and states a `law`,
///   and `addable`; the `type_set` `whole`; `satisfies` lines, one now and then written `.|.`;
///   a struct with its own `below`, `==` and `+`; `gmax`, `gsame`, `gsum`, `gtwice` and
///   `gkeep` called with numbers bare and the struct lent, inside maths and inside each other;
///   `gbox<?item: any.type>` holding an integer; `gpair` and `gfirst`, with two type
///   parameters each, holding an integer and an integer or a struct; and `glarger` with a unit
///   parameter;
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
///   by `str.from` from text or a number, ended by a deferred `str.done`; a struct with a `str`
///   field holding a literal; a function taking `str` views, which pass with no word, and one
///   returning a borrow of one of two, stored as `Lend from <$a, $b> str`; a number into a
///   string with `as`, its failure handled with `??` where it can fail;
pub const NOT_GENERATED: &[&str] = &[
    "1.13 undecided: what a point is, for a width (12.2); `places:` or `form:` with `.stored`; a width with no `align:`",
    "2.6, 2.7 undecided: a negative exponent (12.2)",
    "2.9 undecided: escapes in string literals and how literals work (12.2), and `==` on `str` (12)",
    "2.10 undecided: a full list's failure named in an `otherwise` arm (12.7)",
    "2.12 undecided: putting, getting and going through a map's items",
    "2.14 undecided: what converts to what beyond numbers into strings and units (12.3)",
    "2.16 undecided: `std`'s units other than `time.seconds` and `angle`'s (12.2)",
    "2.18 undecided: maths and `==` between two of the same `type.new` (12)",
    "2.21 NotBuiltYet: bounded lanes, as in `simd<u8[min: 1, max: 16]>`; needs a target: `lanes: max`, whose count is the target's; undecided: maths between a `simd` and one value",
    "2.22 NotBuiltYet: a `simd` of secrets; still to do: secrets in growable lists; undecided: rotating an integer, which has no spelling, and `shr` on a negative (12)",
    "5.x undecided: specialization, value parameters, type slots and how laws are proven (12.5)",
    "9.x undecided: `pub` and `use` between packages, since how a kit of your own is found isn't decided (12.9); NotBuiltYet: channels, waiting on the run loop's rewrite, and `.at(address:)`, which points at hardware the oracle doesn't have; undecided: a shield's own `requires` and `ensures` (12.9), your own allocator (2.10)",
    "10.x compiler-only: build modes, since the oracle runs one way and the live comparison runs every mode against it",
];
