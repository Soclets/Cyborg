# 5. Functions

| Section | What | In short |
|---|---|---|
| [5.1](#51-declaring) | `fn name [ ] -> T:` | `[ ]` describes the parameters |
| [5.2](#52-calling) | `name( )` | `( )` runs it |
| [5.3](#53-return) | `return` | |
| [5.4](#54-the-top-of-the-body) | `requires`, `ensures`, `fails` | In that order, first in the body |
| [5.5](#55-contracts) | `requires`, `ensures` | When they're checked |
| [5.6](#56-methods-) | `.%` | A function on a receiver |
| [5.7](#57-builtins-) | `@size_of` | Bare `@` |
| [5.8](#58-generics) | `<?T: ordered>` | C++ templates, but safer |
| [5.9](#59-pure-functions) | `fn.pure` | Same input, same output, nothing else changes |
| [5.10](#510-the-inline-compile-time-language) | Inline language | Templates, but more powerful and safe |
| [5.11](#511-baking) | `bake`, `fn.pure.baked`, `const` | Compile-time values |

## 5.1 Declaring

**Rule:** a function is declared with `[ ]`, and its parameters are lent unless they say otherwise ([6.1](6-ownership.md#61-the-words)).

```
fn total [$xs: std::list<i32>[len.max: inf]] -> i32:
    …
end fn total
```

**From:** D2, D23.

## 5.2 Calling

**Rule:** a call uses `( )`, and the ownership word goes on the argument. Values like integers take no word.

```
let $t: i32 = total(Lend $xs);
use_score($n);
```

**From:** D2, D20, D23.

## 5.3 `return`

**Rule:** `return` hands back a value. `main` says `return void;`. Whether every `void` function must isn't decided.

**From:** D11, the sketch.

## 5.4 The top of the body

**Rule:** `requires`, `ensures` and `fails` go first in the body, in that order, then the code. So the top of a body says what the function promises.

```
fn half [$n: i32] -> i32:
    requires $n at_least 0;
    ensures result at_least 0;
    fails [odd];

    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half
```

`result` is a placeholder.

**From:** D52, D53, D84.

## 5.5 Contracts

**Rule:** `requires` and `ensures` are in the base language. They're checked at runtime unless Precog proves them, following the build mode ([10.3](10-build-modes.md#103-what-gets-checked)).

Contracts can only call pure functions ([5.9](#59-pure-functions)), so turning checks off can't change what a correct program does.

**From:** D14, D29, D39, D53, D84, D86.

## 5.6 Methods: `.%`

**Rule:** a method is a function attached to a type, called on a receiver with `.%`. A `.` never runs code, and a `.%` always does.

```
$xs.%append(val: 4);
```

Methods only borrow, because `%` can't hide `Give` ([6.8](6-ownership.md#68--unsaid-ownership)). Anything that consumes its value is a plain function: `close(Give $f)`.

How a method is declared isn't decided ([12.5](12-not-decided.md#125-functions)).

**From:** D4, D5, D8.

## 5.7 Builtins: `@`

**Rule:** a bare `@`, with no `::`, is a compiler builtin, and it's always available. A builtin looks like a function call, and can't be written as a Cyborg function.

```
@size_of<i32>()
```

**Refused:** `@let`. `let`, `+` and `as` aren't calls, and `std::println` could be written in Cyborg.

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
end fn area
```

**Refused:** inside one, I/O, changing anything outside the function, `Lent.mut`, calling a function that isn't pure, shields and C calls.

Whether one may allocate or fail isn't decided ([12.5](12-not-decided.md#125-functions)).

**From:** D86.

## 5.10 The inline compile-time language

**Rule:** Cyborg's equivalent of C++ templates is a separate compile-time language, written inline in Cyborg code, more powerful than templates and safe. [5.8](#58-generics) says what its generics must do.

Safe means that if compile-time code passes the compiler, every program it can generate is valid Cyborg, with all of Cyborg's rules:

- It's checked at the definition, not at a use.
- Generated code follows ownership, `%`, shields and everything else.
- It always finishes, such as through a step limit that gives `refused:`.
- Errors point at real code, with `because:` leading back to the compile-time code.

It can pick a type by a condition, take any number of type parameters, and loop over a struct's fields, with each field's type fitting a bound. Generating a struct's fields and generating functions are goals, if the checking works out.

Its syntax isn't designed ([12.5](12-not-decided.md#125-functions)).

**Refused:** generating arbitrary code from text.

**From:** D85, D88, D89, D90.

## 5.11 Baking

**Rule:** all compile-time work belongs to the inline language ([5.10](#510-the-inline-compile-time-language)). A `const`'s value is inline-language code, and `bake` is the shortest piece of it. It can call normal functions marked `fn.pure.baked`, and only those, and they also run normally.

```
fn.pure.baked seconds_in [$days: i32] -> i32:
    return $days x 86_400;
end fn seconds_in

const $week: i32 = seconds_in(7);
let $day: i32 = bake seconds_in(1);
let $n: i32 = seconds_in($d);
```

**Refused:** baking a function that isn't `fn.pure.baked`, and a compile-time step limit being reached. An edit that stops a function being baked is refused on that function.

**From:** D86, D89, D91.
