# 5. Functions

| Section | What | In short |
|---|---|---|
| [5.1](#51-declaring) | `fn name [ ] -> T:` | `[ ]` describes the parameters |
| [5.2](#52-calling) | `name( )` | `( )` runs it |
| [5.3](#53-return) | `return` | |
| [5.4](#54-the-top-of-the-body) | `labels`, `defaults`, `requires`, `ensures` | In that order, first in the body |
| [5.5](#55-contracts) | `requires`, `ensures` | When they're checked |
| [5.6](#56-methods-) | `.%` | A function on a receiver |
| [5.7](#57-builtins-) | `@size_of`, `@fmt` | Bare `@`: what the language defines |
| [5.8](#58-generics) | `<?item: ordered>`, `interface`, `satisfies` | C++ templates, but safer |
| [5.9](#59-pure-functions) | `fn.pure` | Same input, same output, nothing else changes |
| [5.10](#510-compile-time-features) | Compile-time features | Templates, but safe, one feature per job |
| [5.11](#511-baking) | `bake`, `fn.pure.baked`, `const` | Compile-time values |

## 5.1 Declaring

**Rule:** a function is declared with `[ ]`, and its parameters are lent unless they say otherwise ([6.1](6-ownership.md#61-the-words)).

```
fn total [$xs: Lent std::coll::list<i32>[len.max: inf items]] -> i32:
    …
end fn total
```

Declarations work in any order, so a function can call one declared later in the file. The linter can enforce an order.

**From:** D2, D23, D120, D275, D287.

## 5.2 Calling

**Rule:** a call uses `( )`, and the ownership word goes on the argument. Values like integers take no word.

```
let $t: i32 = total(Lend $xs);
use_score($n);
```

A parameter can have a label, declared at the top of the body ([5.4](#54-the-top-of-the-body)), and then every call writes it. A parameter with no label is called without one. Labels stay in their declared order, unless declared `labels.any_order`. A label with a default can be left off.

```
first_word(Lend $text, limit: 20);
first_word(Lend $text);
append(val: Give $r);
```

In `std`, the first argument, the thing being worked on, stays bare, and every other argument gets a label, except lists where every item has the same role, like `print`'s pieces.

**Refused:** leaving off a label with no default, writing a label that wasn't declared, and `labels.in_order`, because it's the default.

**From:** D2, D20, D23, D113, D114.

### The pipe: `|>`

`|>` passes a value along a chain of calls, left to right: each step's result is the next call's bare first argument. The starting value writes its ownership word, and later steps pass fresh results. A step with no other arguments is bare, a step with labels keeps its brackets, and a step that can fail carries its handling (D446).

```
use std::math::sqrt;

let $root: f64.stable = Lend $xs |> average |> sqrt;
let $cfg: config = Lend $path |> load? |> validate ?? defaults();
let $hp: i32 = $hp |> heal(by: 30) |> clamp(max: 100);
```

**From:** D446.

## 5.3 `return`

**Rule:** `return` hands back a value, and every path through a function ends in one. A `return` may end any path, partway or at the end, wherever nothing else can run after it: `return void;` in a `void` function, `main` included, and `return 1000;` in an `-> i32` one. `end fn` only closes the syntax.

```
fn greet [$out: Lent.mut os::io::stdout.buffered, $name: str] -> void:
    if $name.len == 0 bytes:
        return void;
    else:
        @std::io::fmt_print(Lend.mut $out, "Hi {$name}{newln}") ?? void;
        return void;
    end if
end fn greet
```

**Refused:** a path that ends without a `return`, and a `return` with more code still to run after it, since that code could never run.

**From:** D11, D393, D431, D448, the sketch.

## 5.4 The top of the body

**Rule:** `labels`, `defaults`, `requires` and `ensures` go first in the body, in that order, then the code. So the top of a body says how to call it, and what it promises. `fails` goes in the header ([8.1](8-failures.md#81-fails)).

```
fn first_word [$text: str, $limit: i32] -> str fails [too_long]:
    labels [$limit as limit];
    defaults [$limit = 40];
    requires $limit at_least 1;

    …
end fn first_word
```

A fuller example of contracts:

```
fn half [$n: i32] -> i32 fails [odd]:
    requires $n at_least 0;
    ensures result at_least 0;

    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half
```

`result` is a placeholder.

**From:** D52, D53, D84, D114, D209.

## 5.5 Contracts

**Rule:** `requires` and `ensures` are in the base language. They're checked at runtime unless they're proven, by `OKAY\n` or Precog, following the build mode ([10.3](10-build-modes.md#103-what-gets-checked)).

Contracts can only call pure functions ([5.9](#59-pure-functions)), so turning checks off can't change what a correct program does.

**From:** D14, D29, D39, D53, D84, D86.

## 5.6 Methods: `.%`

**Rule:** a method is a function attached to a type, called on a receiver with `.%` when it lends the receiver read-only, or `.%%` when it lends it to change. A `.` never runs code, and a `.%` always does. A method from a kit says its full path, its family's path plus its name, because bare means base or your own, and a `use` shortens it ([1.11](1-programs.md#111-short-names-use)). A limb's has `@`.

```
$xs.std::coll::list.%%append(val: 4) ?? crash;

use std::coll::list.%%append;
$xs.%%append(val: 4) ?? crash;
```

Methods only borrow, because `%` and `%%` stand only for `Lend` and `Lend.mut` ([6.8](6-ownership.md#68--and--lending-a-receiver)). Anything that consumes its value is a plain function: `close(Give $f)`.

```
$p.%area();
```

**Refused:** `$xs.%%append(val: 4);` with no `use`, with `fix: $xs.std::coll::list.%%append(val: 4);`; `$xs.std::%%append(val: 4);`, with the same fix; and `$xs.std::coll::list.%append(val: 4);`, with `fix: .%%`, since `append` changes its receiver.

A type's methods are declared together in a `methods` block named after the type. Each is a normal function inside it, and its first parameter is the receiver, which says how it's held, matching the `%` or `%%` in its name. A method isn't a family member, since warming up isn't a kind of reading ([1.6](1-programs.md#16-narrowings-)).

```
methods reading:
    fn %%warm_up [$r: Lent.mut reading, $by: i32] -> void:
        labels [$by as by];
        change $r.celsius = $r.celsius + $by;
        return void;
    end fn %%warm_up
end methods reading

let mut $r: reading = reading[celsius = 20];
$r.%%warm_up(by: 2);
```

A family holds only a word and its kinds, so a `fam.fn` holds a function and its kinds, as `print` and `print.stored`, and related functions that aren't kinds of one another go in a module, as `geometry::area` ([1.5](1-programs.md#15-paths-)). Where a type's makers, like `.new` and `.from`, are declared isn't decided ([12.5](12-not-decided.md#125-functions)).

**Refused:** a receiver that's `Given`, because methods only borrow; a method in a `methods` block not named after its receiver's type; and a `fam.fn` member that isn't a kind of the family's name, as `area` in `fam.fn geometry`, with a fix moving it to a module.

**From:** D4, D5, D8, D115, D130, D135, D211, D423, D480, D481.

## 5.7 Builtins: `@`

**Rule:** a bare `@`, with no `::`, is a compiler builtin, and it's always available. A builtin looks like a function call, and can't be written as a Cyborg function.

A builtin is for what the language itself defines. A type's size and alignment, fixed by the standard for every type, base types included, are `@size_of<player>()` and `@align_of<player>()`, in `bytes` ([9.8](9-limbs-and-shields.md#98-what-must-live-in-a-shield)). `@fmt("…")` makes a format value, of type `fmt`: the text, checked while building, and borrows of the values its holes read, with no text made yet, for a kit that takes a format, like a logger taking `$f: Lent fmt`. Stored in a variable, it names what it borrows, as `Lend from <$name, $count> fmt` ([6.10](6-ownership.md#610-stored-borrows)). Conveniences built on top live in `std`, like printing, in its limb half ([1.13](1-programs.md#113-printing-stdiofmt_print)).

```
@size_of<i32>()
@align_of<player>()
@fmt("{$name}: {$count}")
```

The rest of the list of builtins isn't decided ([12.9](12-not-decided.md#129-limbs-and-projects)).

**Refused:** `@let`. `let`, `+` and `as` aren't calls.

A `fmt` becomes text as a number does: `@fmt("{$name}: {$count}") as str[len.max: 64 bytes]` into a bounded string, failing if it doesn't fit, or `str.from(@fmt("{$name}: {$count}"), alloc: Lend $heap)` into a growable one ([2.14](2-values-and-types.md#214-conversion-as)).

**From:** D56, D328, D369, D454, D461, D467, D468.

## 5.8 Generics

**Rule:** a generic function or type makes a copy per type, which runs as fast as hand-written code. What it needs from a type parameter is a requirement, and it's checked once, at its definition. It can take values as parameters, can be specialized for one type, and can be extended by outsiders through requirements.

```
fn largest<?item: ordered> [$a: Lent ?item, $b: Lent ?item] -> ?item:
    if $a above $b:
        return Copy $a;
    end if
    return Copy $b;
end fn largest

struct pair<?left: any.type, ?right: any.type>:
    first: ?left;
    second: ?right;
end struct pair

let $x: i32 = largest(3, 5);
let $r: reading = largest(Lend $morning, Lend $noon);
```

A type parameter is `?` and a lowercase name, declared in `< >` straight after the name, and always writes its requirement. A call fills each type parameter from its arguments, and when they're bare literals, from the type the result goes into, so `?item` above is `i32`. If nothing says a type, it's refused. `any.type` means it needs nothing, so the body can only store, move, copy and lend it. A type parameter takes ownership words like a struct, since it might be filled by one, and they do nothing for a plain number: a parameter of type `?item` says `Lent`, `Lent.mut` or `Given`, while a call writes the word its actual argument needs, so numbers go bare and structs take `Lend`. When a requirement can only be filled by plain numbers, like a `type_set` of `i32` and `i64`, there's no choice, so the parameter is bare, as in `smaller` below. A unit parameter writes `any.unit`, or a family's name, and goes only where a unit goes ([2.16](2-values-and-types.md#216-units)).

A requirement is one of two kinds, never mixed. An `interface` lists operations a type must have, with no code, and writes their parameters' ownership words as any signature does, and can state laws, include another interface, and leave type slots. A `type_set` is a closed list of types:

```
interface ordered<?self>:
    includes equatable;
    op below [$a: Lent ?self, $b: Lent ?self] -> bool;
    law if $a below $b and $b below $c then $a below $c;
end interface ordered

type_set whole:
    i32;
    i64;
end type_set whole

reading satisfies ordered;

fn smaller<?item: whole and ordered> [$a: ?item, $b: ?item] -> ?item:
```

Requirements combine with `and`. Only an interface gives operations, so a body can't use `+` on a `?item` whose requirement is only a type set: it writes `<?item: whole and addable>`. `std` declares which interfaces its own types satisfy. A type fits an interface only through a `satisfies` line, which only the type's package or the interface's package can write. `reading .|. ordered;` means exactly the same: an easter egg, on purpose.

"Bound" means a value's min and max ([2.5](2-values-and-types.md#25-bounded-integers)), so these are requirements, not bounds. How laws are proven, and how a type slot is written, aren't decided ([12.5](12-not-decided.md#125-functions)). A type's own operators are in [3.10](3-operators.md#310-operators-for-your-own-types-op).

**Refused:** a parameter of a type parameter that a struct could fill, with no ownership word, with a fix asking which; a type parameter with no requirement, with `fix: ?left: any.type`, a bare `any`, asking which kind, a body that uses a `?item` in a way its requirement doesn't list, a type used where it hasn't said `satisfies`, a `satisfies` line written outside the type's package and the interface's package, and an interface and a type set mixed in one.

**From:** D23, D85, D87, D89, D234, D235, D238, D239, D240, D241, D243, D244, D394, D395, and Radium's generics.

## 5.9 Pure functions

**Rule:** a pure function always gives the same output for the same input, and changes nothing else.

```
fn.pure area [$w: i32, $h: i32] -> i32:
    return $w x $h;
end fn.pure area
```

**Refused:** inside one, I/O, changing anything outside the function, `Lent.mut`, calling a function that isn't pure, shields and C calls.

Whether one may allocate or fail isn't decided ([12.5](12-not-decided.md#125-functions)).

**From:** D86.

## 5.10 Compile-time features

**Rule:** Cyborg has no separate compile-time language for now. Each compile-time job is its own feature:

| Job | Feature |
|---|---|
| Picking a type by a condition | A `match` on types inside a generic |
| Any number of type parameters | Variadic generics |
| Looping over a struct's fields | A built-in loop over a type's fields, each field's type fitting a bound |
| Values known while compiling | `const`, `bake` and `fn.pure.baked` ([5.11](#511-baking)) |

Each one is safe: if it passes the compiler, every program it can produce is valid Cyborg, with all of Cyborg's rules.

- It's checked at the definition, not at a use.
- Produced code follows ownership, `%`, shields and everything else.
- It always finishes, such as through a step limit that gives `refused:`.
- Errors point at real code, with `because:` leading back to the feature.

Generating whole functions or structs waits. A separate compile-time language could come back later as a limb. Each feature's syntax isn't designed ([12.5](12-not-decided.md#125-functions)).

**Refused:** generating arbitrary code from text.

**From:** D85, D88, D89, D90, D217, D218.

## 5.11 Baking

**Rule:** a value known while compiling comes from `const` or `bake` ([5.10](#510-compile-time-features)). A `const`'s value is worked out while compiling, and `bake` works out one value in place. It can call normal functions marked `fn.pure.baked`, and only those, and they also run normally.

```
fn.pure.baked seconds_in [$days: i32] -> i32:
    return $days x 86_400;
end fn.pure.baked seconds_in

const $week: i32 = seconds_in(7);
let $day: i32 = bake seconds_in(1);
let $n: i32 = seconds_in($d);
```

**Refused:** baking a function that isn't `fn.pure.baked`, and a compile-time step limit being reached. An edit that stops a function being baked is refused on that function.

**From:** D86, D89, D91, D218.
