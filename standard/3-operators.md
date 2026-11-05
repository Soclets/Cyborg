# 3. Operators

| Section | What | In short |
|---|---|---|
| [3.1](#31-every-operator) | All of them | One table |
| [3.2](#32-arithmetic----x) | `+`, `-`, `x` | Overflow depends on the mode |
| [3.3](#33-division-div) | `div.trunc`, `div.floor`, `div.exact` | No bare `div` |
| [3.4](#34-remainder-mod) | `mod` | Remainder |
| [3.5](#35-comparison) | `==`, `!=`, `<`, `>`, `at_least`, `at_most` | |
| [3.6](#36-logic) | `and`, `or`, `not` | |
| [3.7](#37-bits-and-shifts) | `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr` | |
| [3.8](#38-wrapping-pluswrap) | `plus.wrap` | Wraps in every mode |
| [3.9](#39-grouping) | `( )` | The one exception to [1.8](1-programs.md#18-brackets) |

## 3.1 Every operator

| Kind | Spelling | Section |
|---|---|---|
| Arithmetic | `+`, `-`, `x` | [3.2](#32-arithmetic----x) |
| Divide | `div.trunc`, `div.floor`, `div.exact` | [3.3](#33-division-div) |
| Remainder | `mod` | [3.4](#34-remainder-mod) |
| Comparison | `==`, `!=`, `<`, `>`, `at_least`, `at_most` | [3.5](#35-comparison) |
| Logic | `and`, `or`, `not` | [3.6](#36-logic) |
| Bits | `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr` | [3.7](#37-bits-and-shifts) |
| Set a value | `=` | [2.1](2-values-and-types.md#21-let), [7.2](7-structs-and-unions.md#72-building-a-struct) |
| Define a constant | `:=` | [2.3](2-values-and-types.md#23-constants-) |
| Convert | `as` | [2.14](2-values-and-types.md#214-conversion-as) |
| Inexact literal | `~` | [2.7](2-values-and-types.md#27-inexact-literals-) |
| Pass a failure up | `?` | [8.4](8-failures.md#84--pass-it-up) |
| Use this instead | `??` | [8.5](8-failures.md#85--use-this-instead) |

Operator precedence isn't decided ([12.3](12-not-decided.md#123-operators-and-conversion)).

**From:** D43, D44, D49, D58, D59.

## 3.2 Arithmetic: `+`, `-`, `x`

**Rule:** `x` is multiply.

```
$a + $b
$a - $b
$a x $b
```

When a plain `+` overflows, what happens depends on the build mode ([10.5](10-build-modes.md#105-overflow)). Arithmetic on a bounded integer gives back a plain `i32` ([2.5](2-values-and-types.md#25-bounded-integers)).

**From:** D20, D42, D43.

## 3.3 Division: `div`

**Rule:** there's no bare `div`, because division has no single right answer for integers. You pick a narrowing.

```
7 div.trunc 2
-7 div.floor 2
8 div.exact 2
```

Those three are examples; the full list isn't decided. What `div.exact` does in each mode isn't decided either. Division by zero traps in `FinalSafe`.

**Refused:** a bare `div`.

**From:** D26, D48.

## 3.4 Remainder: `mod`

**Rule:** `mod` is the remainder.

```
$n mod 2
```

What it gives for negative numbers isn't decided ([12.3](12-not-decided.md#123-operators-and-conversion)).

**From:** D49.

## 3.5 Comparison

**Rule:** `==`, `!=`, `<` and `>` are symbols, and `at_least` and `at_most` are words. There is no `<=` or `>=`. A `<` or `>` comparison has a space on each side.

```
if $t at_least 10:
if $a < $b:
```

**Refused:** `$a<$b`.

**From:** D23, D43, D44, D49.

## 3.6 Logic

**Rule:** `and`, `or` and `not` are words.

**From:** D49.

## 3.7 Bits and shifts

**Rule:** `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl` and `shr` are words.

**From:** D49.

## 3.8 Wrapping: `plus.wrap`

**Rule:** a wrapping operator wraps in every mode, including `Working`. `plus.wrap` is a placeholder spelling.

```
$a plus.wrap $b
```

**From:** D42.

## 3.9 Grouping

**Rule:** `( )` groups in maths, and it's the one case where `( )` doesn't run something.

```
($a + $b) x 2
```

**From:** D23.
