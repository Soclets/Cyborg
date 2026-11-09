# 5. Functions

| Section | What | In short |
|---|---|---|
| [5.1](#51-declaring) | `fn name [ ] -> T:` | `[ ]` describes the parameters |
| [5.2](#52-calling) | `name( )` | `( )` runs it |
| [5.3](#53-return) | `return` | |
| [5.4](#54-the-top-of-the-body) | `labels`, `defaults`, `requires`, `ensures` | In that order, first in the body |
| [5.5](#55-contracts) | `requires`, `ensures` | When they're checked |
| [5.6](#56-methods-) | `.%` | A function on a receiver |
| [5.7](#57-builtins-) | `@bytes_of` | Bare `@` |
| [5.8](#58-generics) | `<?T: ordered>` | C++ templates, but safer |
| [5.9](#59-pure-functions) | `fn.pure` | Same input, same output, nothing else changes |
| [5.10](#510-compile-time-features) | Compile-time features | Templates, but safe, one feature per job |
| [5.11](#511-baking) | `bake`, `fn.pure.baked`, `const` | Compile-time values |

## 5.1 Declaring

**Rule:** a function is declared with `[ ]`, and its parameters are lent unless they say otherwise ([6.1](6-ownership.md#61-the-words)).

```
fn total [$xs: std::list<i32>[len.max: inf items]] -> i32:
    …
end fn total
```

Declarations work in any order, so a function can call one declared later in the file. The linter can enforce an order.

**From:** D2, D23, D120.

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

## 5.3 `return`

**Rule:** `return` hands back a value. `main` says `return void;`. Whether every `void` function must isn't decided.

**From:** D11, the sketch.

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

**Rule:** `requires` and `ensures` are in the base language. They're checked at runtime unless Precog proves them, following the build mode ([10.3](10-build-modes.md#103-what-gets-checked)).

Contracts can only call pure functions ([5.9](#59-pure-functions)), so turning checks off can't change what a correct program does.

**From:** D14, D29, D39, D53, D84, D86.

## 5.6 Methods: `.%`

**Rule:** a method is a function attached to a type, called on a receiver with `.%` when it lends the receiver read-only, or `.%%` when it lends it to change. A `.` never runs code, and a `.%` always does. A method from a kit says its path, because bare means base or your own, and a limb's has `@`.

```
$xs.std::%%append(val: 4);
```

Methods only borrow, because `%` and `%%` stand only for `Lend` and `Lend.mut` ([6.8](6-ownership.md#68--and--lending-a-receiver)). Anything that consumes its value is a plain function: `close(Give $f)`.

```
$p.%area();
$x.@csup::%name();
```

**Refused:** `$xs.%%append(val: 4);`, with `fix: $xs.std::%%append(val: 4);`, and `$xs.std::%append(val: 4);`, with `fix: .%%`, since `append` changes its receiver.

A method is declared as a normal function with its type in the name. The first parameter is the receiver, and says how it's held, which has to match the `%` or `%%` in the name.

```
fn reading.%%warm_up [$r: Lent.mut reading, $by: i32] -> void:
    labels [$by as by];
    change $r.celsius = $r.celsius + $by;
    return void;
end fn reading.%%warm_up

$r.%%warm_up(by: 2);
```

A receiver that's `Given` is refused, because methods only borrow.

**From:** D4, D5, D8, D115, D130, D211.

## 5.7 Builtins: `@`

**Rule:** a bare `@`, with no `::`, is a compiler builtin, and it's always available. A builtin looks like a function call, and can't be written as a Cyborg function.

```
@bytes_of<i32>()
```

**Refused:** `@let`. `let`, `+` and `as` aren't calls, and `std::io::fmt_print` could be written in Cyborg.

**From:** D56.

## 5.8 Generics

**Rule:** a generic function or type makes a copy per type, which runs as fast as hand-written code. It says what it needs through a bound, and is checked once at its definition. It can take values as parameters, can be specialized for one type, and can be extended by outsiders through bounds.

```
fn largest <?T: ordered> [$a: ?T, $b: ?T] -> ?T:
    if $a > $b:
        return $a;
    end if
    return $b;
end fn largest

let $x: i32 = largest(3, 5);
```

`?T` and `ordered` are placeholders. How much of this is spelled isn't decided ([12.5](12-not-decided.md#125-functions)).

A type fits a bound only if it declares that it does, as in `reading is ordered`, which is a placeholder spelling.

**Refused:** a body that uses `?T` in a way its bound doesn't allow, and a type that hasn't declared the bound.

**From:** D23, D85, D87.

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
