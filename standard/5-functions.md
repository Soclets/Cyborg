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

    if $n mod 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half
```

`result` is a placeholder.

**From:** D52, D53, D84.

## 5.5 Contracts

**Rule:** `requires` and `ensures` are in the base language. They're checked at runtime unless Precog proves them, following the build mode ([10.3](10-build-modes.md#103-what-gets-checked)).

**From:** D14, D29, D39, D53, D84.

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
