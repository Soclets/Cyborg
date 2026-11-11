# 3. Operators

| Section | What | In short |
|---|---|---|
| [3.1](#31-every-operator) | All of them | One table |
| [3.2](#32-arithmetic----x) | `+`, `-`, `x` | Overflow depends on the mode |
| [3.3](#33-division-div) | `div.trunc`, `div.floor`, `div.exact`, `div` | No bare `div` on integers |
| [3.4](#34-remainder-mod) | `mod.trunc`, `mod.floor` | No bare `mod` |
| [3.5](#35-comparison) | `==`, `!=`, `below`, `above`, `at_least`, `at_most` | |
| [3.6](#36-logic) | `and`, `or`, `not` | |
| [3.7](#37-bits-and-shifts) | `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr` | |
| [3.8](#38-wrapping-and-saturating-pluswrap-plussaturate) | `plus.wrap`, `plus.saturate` | Wraps or sticks, in every mode |
| [3.9](#39-grouping-and-precedence) | `( )`, precedence | Follows maths |
| [3.10](#310-operators-for-your-own-types-op) | `op below` | A struct defines an operator for itself |

## 3.1 Every operator

| Kind | Spelling | Section |
|---|---|---|
| Arithmetic | `+`, `-`, `x` | [3.2](#32-arithmetic----x) |
| Divide | `div.trunc`, `div.floor`, `div.exact`, and bare `div` on floats | [3.3](#33-division-div) |
| Remainder | `mod.trunc`, `mod.floor` | [3.4](#34-remainder-mod) |
| Comparison | `==`, `!=`, `below`, `above`, `at_least`, `at_most` | [3.5](#35-comparison) |
| Logic | `and`, `or`, `not` | [3.6](#36-logic) |
| Bits | `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr` | [3.7](#37-bits-and-shifts) |
| Set a value | `=` | [2.1](2-values-and-types.md#21-let), [7.2](7-structs-and-unions.md#72-building-a-struct) |
| Convert | `as` | [2.14](2-values-and-types.md#214-conversion-as) |
| Inexact literal | `~` | [2.7](2-values-and-types.md#27-inexact-literals-) |
| Pass a failure up | `?` | [8.4](8-failures.md#84--pass-it-up) |
| Use this instead | `??` | [8.5](8-failures.md#85--use-this-instead) |

Precedence is in [3.9](#39-grouping-and-precedence).

**From:** D43, D44, D49, D59.

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

**Rule:** an integer has no bare `div`, because integer division has no single right answer. You pick a narrowing. A float has a bare `div`, which rounds to the nearest float like `+`, `-` and `x`, so `7.0 div 2.0` is `3.5`.

```
7 div.trunc 2
-7 div.floor 2
8 div.exact 2
```

Those three are examples; the full list isn't decided. What `div.exact` does in each mode isn't decided either. Division by zero traps in `FinalSafe`.

**Refused:** a bare `div` on integers.

**From:** D26, D48, D164.

## 3.4 Remainder: `mod`

**Rule:** there's no bare `mod`, because a remainder has no single right answer for negative numbers. You pick a narrowing, and each one matches its division.

```
-7 mod.trunc 2
-7 mod.floor 2
```

The first gives -1, and the second gives 1.

**Refused:** a bare `mod`.

**From:** D49, D97.

## 3.5 Comparison

**Rule:** equality is symbols, `==` and `!=`, and ordering is words, `below`, `above`, `at_least` and `at_most`, so its direction is spelled out. There is no `<`, `>`, `<=` or `>=` comparison: `<` and `>` only mean brackets.

```
if $t at_least 10:
if $a below $b:
```

**Refused:** `$a < $b`, with `fix: below`, and `$a > $b`, with `fix: above`.

**From:** D23, D43, D44, D49, D278.

## 3.6 Logic

**Rule:** `and`, `or` and `not` are words. `and` and `or` stop early, so the right side only runs if it's needed. A narrowing runs both sides, spelled `and.both` and `or.both` as placeholders.

```
if $n != 0 and 10 div.trunc $n above 1:
```

**From:** D49, D124.

## 3.7 Bits and shifts

**Rule:** `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl` and `shr` are words.

**From:** D49.

## 3.8 Wrapping and saturating: `plus.wrap`, `plus.saturate`

**Rule:** `+`, `-` and `x` are written as words, `plus`, `minus` and `times`, followed by how they handle overflow, the same shape as `div.trunc`. Plain `+`, `-` and `x` stay the checked defaults.

```
$a plus.wrap $b       $a plus.saturate $b
$a minus.wrap $b      $a minus.saturate $b
$a times.wrap $b      $a times.saturate $b
```

A wrapping operator wraps in every mode, including `Working`.

A saturating operator sticks at the type's own bounds instead of overflowing, including a written `min:` and `max:`, on one value or on every lane of a `simd` (D291). On a type with no written bounds, like `u8`, those are the hardware's, so it's one native instruction; written bounds add a `min` and a `max`. 
```
let $volume: u8[min: 0, max: 100] = 95;
let $louder: u8[min: 0, max: 100] = $volume plus.saturate 10;
```

`$louder` is 100. To stick at the hardware's limit and check a bound afterwards, saturate in the plain type first, then assign.

**Refused:** `+.wrap` and other narrowings on a symbol, with `fix: plus.wrap`, and a bare `plus`, `minus` or `times`, with `fix: +`, `-` or `x`.

**From:** D42, D294, D295.

## 3.9 Grouping and precedence

**Rule:** precedence follows maths, and `( )` groups, which is the one case where `( )` doesn't run something. Tightest first:

| Level | Operators |
|---|---|
| 1 | `.`, `.%`, `::`, calls, `?`, `??` |
| 2 | Unary `-` |
| 3 | `x`, `div…`, `mod…`, left to right |
| 4 | `+`, `-` |
| 5 | `==`, `!=`, `below`, `above`, `at_least`, `at_most` |
| 6 | `not` |
| 7 | `and` |
| 8 | `or` |

```
$a + $b x 2
($a + $b) x 2
($a + $b) as str[len.max: 20 bytes]
$a + read() ?? 0
```

**Refused:** where maths gives no order, so bit operators and shifts mixed with anything, as in `$a shl 2 + 1`, and `as` on anything but a single value, as in `$a + $b as str[len.max: 20 bytes]`.

Unary `-` negates, so `-$x` and `-7` are both allowed. A comparison chain means all of it is true, and each value is evaluated once: `1 below $x below 5` means `1 below $x and $x below 5`. It stops at its first false. The linter can ask for parentheses.

**From:** D23, D116, D117, D124, D131, D132.

## 3.10 Operators for your own types: `op`

**Rule:** a struct or union can define an operator for itself, as a function named by the operator. It keeps the operator's usual precedence, and applies only where its parameters' types match. This is how a type gets what an interface asks for ([5.8](5-functions.md#58-generics)).

```
op below [$a: reading, $b: reading] -> bool:
    return $a.celsius below $b.celsius;
end op below

reading satisfies ordered;
```

A type can define arithmetic, `+`, `-`, `x` and the `div` narrowings, and comparisons. Only `==` and `below` are written: `!=`, `above`, `at_least` and `at_most` follow from them, so they can never disagree.

**Refused:** defining or changing an operator on a built-in type, like `i32`, writing `!=`, `above`, `at_least` or `at_most` yourself, and defining `and`, `or`, `not`, `=`, `?`, `??` or `as`.

**From:** D116, D117, D124, D235, D236, D237, and CyborgPL's `op`.
