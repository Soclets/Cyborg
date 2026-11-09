# 2. Values and types

| Section | What | In short |
|---|---|---|
| [2.1](#21-let) | `let` | A variable that can't change |
| [2.2](#22-let-mut-and-change) | `let mut`, `change` | A variable that can |
| [2.3](#23-constants-const) | `const` | Known at compile time |
| [2.4](#24-integers) | `i32`, `i64` | One spelling, no chain |
| [2.5](#25-bounded-integers) | `i32[min: , max: ]` | A range the compiler holds you to |
| [2.6](#26-floats) | `f32`, `f64` | Binary floats |
| [2.7](#27-inexact-literals-) | `~` | This literal gets rounded |
| [2.8](#28-decimal-floats) | `d32` | A limb, likely |
| [2.9](#29-strings) | `str` | UTF-8, or pick another |
| [2.10](#210-lists) | `std::list` | 0 to `len.max` items |
| [2.11](#211-arrays) | `std::array` | Exactly `len` items |
| [2.12](#212-maps) | `std::map` | Labels required |
| [2.13](#213-labels-in--) | Labels in `< >` | One bare, more labeled |
| [2.14](#214-conversion-as) | `as` | Builds a new value |
| [2.15](#215-void-and-bool) | `void`, `bool` | |
| [2.16](#216-units) | `3600 time.seconds` | A number says its unit |
| [2.17](#217-getting-an-item) | `$xs[pos: 1]`, `$xs[offset.items: 0]` | Counting from 1, or from 0 |
| [2.18](#218-naming-a-type) | `type.alias`, `type.new` | Another name, or a different type |

## 2.1 `let`

**Rule:** `let` declares a variable with its type and value, and it can't change.

```
let $t: i32 = total(Lend $xs);
```

**From:** the sketch.

## 2.2 `let mut` and `change`

**Rule:** `let mut` declares a variable that can change, and `change` reassigns it.

```
let mut $sum: i32 = 0;
change $sum = $sum + $x;
```

`change` only reassigns, a variable or one field of it ([7.3](7-structs-and-unions.md#73-reading-a-field)). It doesn't stand in for `Lend.mut`.

**From:** D3, D125, the sketch.

## 2.3 Constants: `const`

**Rule:** `const` declares a constant, with no `let`. Its value must be fully known at compile time.

```
const $b: i32 = 60 x 60;
const $pi: f64 = ~3.14159265358979323846;
```

It can call `fn.pure.baked` functions ([5.11](5-functions.md#511-baking)).

**Refused:** `const $a: i32 = read();`.

**From:** D58, D59, D83, D91.

## 2.4 Integers

**Rule:** a 32-bit signed integer is `i32`. There is one spelling and no alias, and `int.signed.32_bit` does not exist. `i64` also appears in the decisions. The full list isn't decided ([12.2](12-not-decided.md#122-types)).

**From:** D10, D57.

## 2.5 Bounded integers

**Rule:** `i32[min: 0, max: 100]` is still 32 bits, and holds only that range.

- A literal the compiler can see fits needs no check.
- When the compiler can't see a value fits, a plain `=`, `change` or argument does a runtime check.
- An out-of-range value crashes, or is UB in a mode with no checks ([10.4](10-build-modes.md#104-what-happens-when-something-goes-wrong)).
- Arithmetic gives back a plain `i32`.
- A bounded value goes into a plain `i32` with no check.
- To handle a bad value yourself, use the test in [4.2](4-control-flow.md#42-the-test-if-n-t).

```
let $a: i32[min: 0, max: 100] = 50;
let $b: i32[min: 0, max: 100] = $n;
change $b = $b + 1;
use_score($n);
```

**Refused:** `let $a: i32[min: 0, max: 100] = 150;`.

**From:** D20.

## 2.6 Floats

**Rule:** a 32-bit binary float is `f32`, and `f64` is 64-bit. A float can have bounds, like an integer. It can't be NaN or infinity: making one is a failure, a runtime check in `FinalSafe`. It gives the same bits on every machine.

```
let $t: f64[min: -40.0, max: 125.0] = 21.5;
```

Narrowings opt out, with placeholder names: `f64.ieee` allows NaN and infinity, and `f64.fast` allows merged and reordered maths. Merged maths can change an answer slightly, or whether a program crashes, and that isn't a bug. `f64.ieee` needs no shield, since it's a different type, and `as` is the wall:

```
let $reading: f64.ieee = read_sensor();
let $clean: f64 = $reading as f64;
```

**Refused:** `let $x: f64 = $reading;`.

Comparing works as usual, and `-0.0 == 0.0` is true. The linter can flag comparing floats with `==`.

IEEE 754 is the authority on how `f64.ieee` behaves, including comparing, arithmetic, and NaN and infinity moving through the maths. So any comparison with NaN is false except `!=`, and `NaN == NaN` is false. Cyborg still decides syntax, printing, `as` crossings and bounds.

A float prints as the shortest text that means its value, and `std::io::fmt_print.stored` prints what's actually in memory:

```
let $x: f64 = ~0.1;
std::io::fmt_print("{}{newln}", $x);
std::io::fmt_print.stored("{}{newln}", $x);
```

The first prints `0.1`, and the second prints every digit, with no limit: `0.1000000000000000055511151231257827021181583404541015625`.

On `f64.ieee`, both print NaN and infinity as the words `NaN`, `inf` and `-inf`.

`std::math::sin` and similar functions are correctly rounded, so they give the same bits on every machine. `std::math::sin.fast` is a placeholder for a version that isn't.

**From:** D54, D59, D158, D159, D160, D161, D162, D163, D166, D167.

## 2.7 Inexact literals: `~`

**Rule:** a float literal the type can't hold exactly must say so with `~`. An exact one has no mark. A negative is `~-0.1`. There's no limit on digits. `~` covers literals only, not arithmetic.

```
let $a: f64 = 0.5;
let $b: f64 = ~0.1;
```

A literal with an exponent says so in front of the value, with `exponent` as a placeholder name. It's still a plain float, and it doesn't remember how it was written, so it prints as plain digits, however long.

```
let $avogadro: f64 = exponent ~6.02214076e23;
```

**Refused:** `let $b: f64 = 0.1;` and `let $a: f64 = ~0.5;`, each with the fix shown, and `~6.02214076e23` without `exponent`.

**From:** D59, D165.

## 2.8 Decimal floats

**Rule:** a decimal float is likely `d32`. It's a limb, not base and not a kit, so it's marked `@`. Which limb is not decided, and it could be a money limb. In `d32`, `0.1` is exact, so it needs no `~`.

**From:** D54, D55, D59.

## 2.9 Strings

**Rule:** a bare `str` means UTF-8. Other encodings are narrowings, such as `str.utf16`.

An empty string is `empty`.

```
let $a: str = "hi";
let $b: str = empty;
```

**Refused:** `str.utf8`, because it's the default ([1.7](1-programs.md#17-bare-names)), and `""`, with `fix: empty`, because it could look like unfinished work.

**From:** D48, D50, D51, D103.

## 2.10 Lists

**Rule:** a list holds its element type in `< >`, and always writes its `len.max`, with `inf` for unbounded. It can also have a `len.min`. An empty list always writes `= empty`.

```
let mut $xs: std::list<i32>[len.max: inf items] = empty;
let mut $ys: std::list<i32>[len.max: 8 items] = empty;
std::list<i32>[len.min: 1 items, len.max: 8 items]
```

`$xs.len` is `u64.items`. A list can choose another type with `len.type`, which also changes how it stores its length:

```
std::list<i32>[len.max: inf items, len.type: u32]
```

`[ ]` gives it items, and the count is checked against the type:

```
let $b: std::list<i32>[len.min: 1 items, len.max: 8 items] = [4, 6];
```

**Refused:** `std::list<i32>` with no `len.max`, a list with a bare `len`, in favour of `array`, `= empty` on a list with `len.min` above 0, `[]`, with `fix: empty`, and too few or too many items.

**From:** D11, D12, D13, D24, D102, D103, D140.

## 2.11 Arrays

**Rule:** an array has exactly `len` items.

`[ ]` gives it items, and `all` fills it with one value.

```
let $a: std::array<i32>[len: 3 items] = [1, 2, 3];
let mut $d: std::array<u8>[len: 4096 items] = all 0;
```

**Refused:** `len.always`, which doesn't exist, the wrong number of items, and `= empty`.

**From:** D12, D13, D24, D102.

## 2.12 Maps

**Rule:** you pick the kind of map, `std::map.hash` or `std::map.ordered`, and its types get labels. Those two names are placeholders.

```
std::map.hash<key: str, val: i32>
```

The label names are a placeholder.

**Refused:** `std::map.hash<str, i32>`, and a bare `std::map`.

**From:** D24, D25, D138.

## 2.13 Labels in `< >`

**Rule:** one type can go bare, and more than one must have labels.

**From:** D25.

## 2.14 Conversion: `as`

**Rule:** `as` builds a new value of another type, which can allocate memory or fail. `:` doesn't convert.

```
let $text: str = $t as str;
```

A failed `as` is a failure, so it crashes unless handled ([8.9](8-failures.md#89-crash-by-default)). What can convert to what isn't decided ([12.3](12-not-decided.md#123-operators-and-conversion)).

A number converts bare, with no unit, so `$xs.len as str` gives `3`. A narrowing asks for the unit: `as str.with_unit`, a placeholder.

**From:** D7, D96, D122.

## 2.15 `void` and `bool`

**Rule:** a function that returns nothing returns `void`. A `bool` is `true` or `false`, is always copied, and takes no ownership word.

**From:** D2, D11, D127.

## 2.16 Units

These examples assume `time` and `storage` are declared with `fam.unit`, further down. Nothing is built in except `items`.

**Rule:** a number says its unit, behind it where possible. Where the unit can't go behind, it goes in front. A unit always says its family, or its path from a kit, and `items` is base language, so it stays bare. The compiler checks units: a unit is part of the type, carried by a narrowing. The same units can add and compare.

```
let $a: i32.time.seconds = 3600 time.seconds;
let $b: i32.time.seconds = 5 time.seconds;
let $c: i32.time.seconds = $a + $b;

use std::units::time.seconds as seconds;
let $d: i32.seconds = 3600 seconds;
```

A unit is required only where the type asks for one. A plain `i32` means just a number. The language and `std` ask for units on things that have them, like lengths, sizes and durations, and your own code opts in with a type like `i32.time.seconds`.

```
std::array<u8>[len: 4096 items]
let $x: i32 = 3600;
```

`as` takes a unit off a value, or puts one on: `$xs.len as u64`, `$n as u64.items`.

A label can carry the unit instead, when the writer declares it that way. A plain number takes the label's unit, and a value in another unit is refused.

```
fn draw [$x: i32.screen.pixels] -> void:
    labels [$x as at.screen.pixels];
    …
end fn draw

draw(at.screen.pixels: 5);
```

**Refused:** mixing units, as in `$a + 5 storage.bytes`, a missing unit, as in `let $b: i32.time.seconds = 3600;`, a bare `3600 seconds` with no `use`, and unit maths with no `together` line.

You define units in a collection. Every unit says what it is: `: base`, or `=` with a conversion, which can point at a unit in any family. Two units convert if they trace back to the same base, and otherwise they don't.

```
fam.unit money:
    money.cents: base;
    money.dollars = 100 money.cents;

    money.euro_cents: base;
    money.euros = 100 money.euro_cents;
end fam.unit money
```

A family is only a group of related names that a generic can ask for. A unit from a kit is written with its path, `name::unit`.

**Refused:** a unit that says neither `base` nor `=`, a loop of `=` lines, and converting between units with different bases.

Multiplying or dividing two units only works where a `together` line declares it, and the line names the result. The spelling is a placeholder.

```
together length.meters / time.seconds = speed.meters_per_second;
```

A conversion can be an offset, not only a multiple. A moment is kept apart from a length of time. How these are spelled, and how a conversion is written, aren't decided ([12.2](12-not-decided.md#122-types)).

**From:** D99, D100, D101, D108, D109, D110, D111, D112, D141, D144.

## 2.17 Getting an item

**Rule:** `pos:` counts from 1, and `offset:` counts from 0, so the word says which you mean. Both work on lists and arrays.

```
let $first: i32 = $xs[pos: 1];
let $same: i32 = $xs[offset.items: 0];
change $xs[pos: 3] = 10;
```

A bad position is a bounds check ([10.4](10-build-modes.md#104-what-happens-when-something-goes-wrong)). Changing an item needs what changing a field needs ([7.3](7-structs-and-unions.md#73-reading-a-field)).

`pos:` and `offset.items:` take a plain number or one in `items`, so `$xs[pos: $xs.len]` gets the last item. `offset:` says its unit in the label, because it could read as bytes.

**Refused:** a bare `offset:`, and a value in another unit.

**From:** D26, D125, D133, D142, D143, D144.

## 2.18 Naming a type

**Rule:** `type.alias` gives a type another name, fully interchangeable. `type.new` makes a different type from the same data, which doesn't mix with others. There's no bare `type`.

```
type.alias numbers = std::list<i32>[len.max: inf items];
type.new user_id = i64;
```

A family of them states its kind once, and they're always used with the family:

```
fam.type.new ids:
    ids.user = i64;
    ids.order = i64;
end fam.type.new ids

let $u: ids.user = get_user();
```

A literal takes its type from where it lands. A value that already has a type crosses with `as`, which can't fail, since it's the same data.

```
let $u: user_id = 5;
let $v: user_id = $n as user_id;
let $m: i64 = $u as i64;
```

**Refused:** a bare `type`, mixing `ids.user` and `ids.order`, mixing the two kinds in one family, and a value crossing with no `as`, as in `let $u: user_id = $n;`.

A struct, list or string in a variable says what happens to it: `Give $r as wrapped`, or `Copy $r as wrapped`. A plain number or a fresh value needs no word.

**From:** D134, D141, D149.
