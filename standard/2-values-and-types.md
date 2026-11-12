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
| [2.19](#219-decl-and-assign) | `decl`, `assign` | A value given later, once |
| [2.20](#220-types-are-always-written-and-precogs-pick-precogset_to_best) | `@precog::set_to_best` | Precog picks the type |
| [2.21](#221-simd) | `simd<i32>[lanes: 4]` | Several values worked on by one instruction |
| [2.22](#222-secrets-secret) | `u32.secret` | A value that can't affect timing |

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
const $pi: f64.stable = ~3.14159265358979323846;
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

**Rule:** a 32-bit binary float is `f32`, and `f64` is 64-bit. Every float type says which kind it is, straight after the type: `.stable` gives the same bits on every machine, `.fast` allows merged and reordered maths, and `.ieee` allows NaN and infinity. A bare `f64` is refused, asking which. A float can have bounds, like an integer. A `.stable` or `.fast` float can't be NaN or infinity: making one is a failure, a runtime check in `FinalSafe`. It gives the same bits on every machine.

```
let $t: f64.stable[min: -40.0, max: 125.0] = 21.5;
```

Narrowings opt out, with placeholder names: `f64.ieee` allows NaN and infinity, and `f64.fast` allows merged and reordered maths. Merged maths can change an answer slightly, or whether a program crashes, and that isn't a bug. `f64.ieee` needs no shield, since it's a different type, and `as` is the wall:

```
let $reading: f64.ieee = read_sensor();
let $clean: f64.stable = $reading as f64.stable;
```

**Refused:** `let $x: f64.stable = $reading;`.

Comparing works as usual, and `-0.0 == 0.0` is true. The linter can flag comparing floats with `==`.

IEEE 754 is the authority on how `f64.ieee` behaves, including comparing, arithmetic, and NaN and infinity moving through the maths. So any comparison with NaN is false except `!=`, and `NaN == NaN` is false. Cyborg still decides syntax, printing, `as` crossings and bounds.

A float prints as the shortest text that means its value, and `std::io::fmt_print.stored` prints what's actually in memory:

```
let $x: f64.stable = ~0.1;
std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $x);
```

The first prints `0.1`, and the second prints every digit, with no limit: `0.1000000000000000055511151231257827021181583404541015625`.

On `f64.ieee`, both print NaN and infinity as the words `NaN`, `inf` and `-inf`.

`std::math::sin` and similar functions are correctly rounded, so they give the same bits on every machine. They come from CORE-MATH, third-party C kept in `std` under rule 1's exception (D161). Its license is MIT, checked in its repository's LICENSE file on 2026-11-10. They ask for an angle in radians, `f64.stable[unit: std::units::angle.radians]`, so a plain number is refused, and degrees are converted with `as` ([2.16](#216-units)). `std::math::sin.fast` is a placeholder for a version that isn't.

**From:** D54, D59, D158, D159, D160, D161, D162, D163, D166, D167, D257.

## 2.7 Inexact literals: `~`

**Rule:** a float literal the type can't hold exactly must say so with `~`. An exact one has no mark. A negative is `~-0.1`. There's no limit on digits. `~` covers literals only, not arithmetic.

```
let $a: f64.stable = 0.5;
let $b: f64.stable = ~0.1;
```

A literal with an exponent says so in front of the value, with `exponent` as a placeholder name. It's still a plain float, and it doesn't remember how it was written, so it prints as plain digits, however long.

```
let $avogadro: f64.stable = exponent ~6.02214076e23;
```

**Refused:** `let $b: f64.stable = 0.1;` and `let $a: f64.stable = ~0.5;`, each with the fix shown, and `~6.02214076e23` without `exponent`.

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

A string's memory works like a list's ([2.10](#210-lists)), with a `len.max` in bytes, since UTF-8 characters vary in size. A finite `len.max` holds the bytes inside the value: no allocator, nothing to free, and too much text is a failure. `len.max: inf bytes` grows on the heap, takes an allocator on each call that grows it, and is `must_Give` ([6.9](6-ownership.md#69-must_give)). It ends with `str.done`, the word that ends every owner ([6.9](6-ownership.md#69-must_give)).

```
let $name: str[len.max: 32 bytes] = "Tankun";
let mut $log: str[len.max: inf bytes] = empty;
defer str.done(Give $log, alloc: Lend $alloc);
```

A bare `str` is a read-only view of text: any length, never owning memory, never changing and never `must_Give`. A literal is one that lasts the whole program, and lending any string gives one, so a function that takes `str` reads every kind of string. A literal never goes away, so a view of one is stored and returned freely, as in a struct field holding `"kitchen"`. A view of another string is a borrow: passing one into a function needs nothing, but storing or returning one writes its source, following the rules for stored borrows ([6.10](6-ownership.md#610-stored-borrows)), as in `fn first_word [$text: str] -> Lend from $text str:`.

```
fn load [$path: str] -> config fails [file_failed, bad_syntax]:
    …
end fn load

load("data.txt");
load(Lend $name);
load(Lend $log);
```

| Type | Owns memory | Can change |
|---|---|---|
| `str` | No, a view | No, though a `let mut` view can point at other text |
| `str[len.max: 32 bytes]` | Yes, inside the value | Yes |
| `str[len.max: inf bytes]` | Yes, on the heap, `must_Give` | Yes |

**Refused:** `str.utf8`, because it's the default ([1.7](1-programs.md#17-bare-names)), and `""`, with `fix: empty`, because it could look like unfinished work.

**From:** D48, D50, D51, D103, D326, D327, D367, D368.

## 2.10 Lists

**Rule:** a list holds its element type in `< >`, and always writes its `len.max`, with `inf` for unbounded. It can also have a `len.min`. An empty list always writes `= empty`.

```
let mut $xs: std::list<i32>[len.max: inf items] = empty;
let mut $ys: std::list<i32>[len.max: 8 items] = empty;
std::list<i32>[len.min: 1 items, len.max: 8 items]
```

`$xs.len` is `u64[unit: items]`. A list can choose another type with `len.type`, which also changes how it stores its length:

```
std::list<i32>[len.max: inf items, len.type: u32]
```

`[ ]` gives it items, and the count is checked against the type:

```
let $b: std::list<i32>[len.min: 1 items, len.max: 8 items] = [4, 6];
```

Allocation is explicit, as in Zig: anything that takes heap memory takes an allocator, the way printing takes a writer ([1.13](1-programs.md#113-printing-stdiofmt_print)). A list with a finite `len.max` needs none, since its size is known. A growable list can't take items straight from `[ ]`, since filling it allocates: `std::list.from([1, 2, 3], alloc: Lend $heap)` does it, and a growable string comes from `str.from("abc", alloc: Lend $heap)`, which takes a string or a number. Running out of memory is a failure, handled like any other, and a program with no allocator, as on bare metal, can't call anything that allocates. `os::heap()` is the operating system's allocator, so bare metal has none.

```
let $alloc: os::heap = os::heap();
let mut $xs: std::list<i32>[len.max: inf items] = empty;
$xs.std::%%append(val: 4, alloc: Lend $alloc);
```

The list doesn't hold its allocator: every call that allocates is handed it, so a list is plain data with no borrow inside, and moves, returns and sits in structs like any other value. The list remembers which allocator it uses, as a small ID, and handing it a different one is a failed check, like a bounds check.

A list that can allocate, one with `len.max: inf items`, is `must_Give` ([6.9](6-ownership.md#69-must_give)), so its memory can't leak: it ends with `std::list.done(Give $xs, alloc: Lend $alloc);`, which frees it with the same allocator. A struct holding one is `must_Give` too. 

An allocator is anything that satisfies the `std::allocator` interface ([5.8](5-functions.md#58-generics)), written in a shield, since it deals in raw pointers:

```
interface allocator:
    fn ?self.%allocate [$bytes: u64[unit: bytes], $align: u64[unit: bytes]] -> ptr.raw.mut<u8> fails [out_of_memory];
    fn ?self.%resize [$p: ptr.raw.mut<u8>, $old: u64[unit: bytes], $new: u64[unit: bytes]] -> bool;
    fn ?self.%free [$p: ptr.raw.mut<u8>, $bytes: u64[unit: bytes]] -> void;
end interface allocator
```

`allocate` gives a block or fails with `out_of_memory`, the failure lists pass on. `resize` tries to change a block where it is and says whether it could, and `free` hands one back.

An arena hands out memory from one big block and frees it all at once, as `std::arena`. It only carves up memory it's given, so it works anywhere, bare metal included. Lists made from it still end with `done`, where freeing costs nothing, so every owner follows one rule. The arena counts what's still alive, and ending it while anything from it is alive is a failed check.

```
let mut $arena: std::arena = std::arena.new(size: 1 mib, alloc: Lend $heap);
defer std::arena.done(Give $arena, alloc: Lend $heap);

let mut $xs: std::list<i32>[len.max: inf items] = empty;
defer std::list.done(Give $xs, alloc: Lend $arena);
```

**Refused:** `[ ]` with items, or a string literal, given straight to a growable list or string, with `fix: std::list.from` or `str.from`; `std::list<i32>` with no `len.max`, a list with a bare `len`, in favour of `array`, `= empty` on a list with `len.min` above 0, `[]`, with `fix: empty`, and too few or too many items.

**From:** D11, D12, D13, D24, D102, D103, D140, D321, D322, D323, D324, D330, D333, D369.

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

**Rule:** you pick the kind of map, `std::map.hash` or `std::map.ordered`, and its types get labels, `key.type:` and `val.type:`, the way a list's length type is `len.type:`. A bare `key:` or `val:` is always a value, as in a call, so the type says `.type`. The two kind names are placeholders.

```
std::map.hash<key.type: str, val.type: i32>
```

A map owns memory, so it's `must_Give` and ends with `std::map.done(Give $m, alloc: Lend $heap)`, like every owner ([6.9](6-ownership.md#69-must_give)).

**Refused:** `std::map.hash<str, i32>`, a bare `std::map`, and `std::map.hash<key: str, val: i32>`, with `fix: key.type:` and `val.type:`.

**From:** D24, D25, D138, D325, D331, D366.

## 2.13 Labels in `< >`

**Rule:** one type can go bare, and more than one must have labels.

**From:** D25.

## 2.14 Conversion: `as`

**Rule:** `as` builds a new value of another type, which can fail but never allocates memory. `:` doesn't convert.

```
let $text: str[len.max: 20 bytes] = $t as str[len.max: 20 bytes];
```

A failed `as` is a failure, so it crashes unless handled ([8.9](8-failures.md#89-crash-by-default)). What can convert to what isn't decided ([12.3](12-not-decided.md#123-operators-and-conversion)).

A number converts bare, with no unit, so `$xs.len as str[len.max: 20 bytes]` gives `3`. A narrowing asks for the unit: `as str.with_unit`, a placeholder.

Text from `as` goes into a bounded `str`, held inside the value, so no allocator is needed. Too much text is a failure, but for numbers the compiler knows the most bytes a type can need, 20 for an `i64` and 11 for an `i32`, so with a big enough bound it can't fail. Heap text comes from a function that takes an allocator, like `str.from($n, alloc: Lend $alloc)`, so every allocation is a visible call.

**Refused:** `as str`, since a view has to point at text that already exists, with `fix: as str[len.max: 20 bytes]` giving the size the type needs, or `fix: Lend $name` when the value is already a string; and `as str[len.max: inf bytes]`, with `fix: str.from`.

**From:** D7, D96, D122, D328.

## 2.15 `void` and `bool`

**Rule:** a function that returns nothing returns `void`. A `bool` is `true` or `false`, is always copied, and takes no ownership word.

**From:** D2, D11, D127.

## 2.16 Units

These examples assume `time` and `storage` are declared with `fam.unit`, further down. Nothing is built in except `items`.

**Rule:** a number says its unit, behind it where possible. Where the unit can't go behind, it goes in front. A unit always says its family, or its path from a kit, and `items` is base language, so it stays bare. The compiler checks units: a unit is part of the type, written as a label in its brackets, next to any bounds, as in `i32[unit: time.seconds]` and `f64.stable[unit: length.meters, min: 0.0 length.meters, max: 1.0 length.kilometers]`. The labels go `unit:`, then `min:`, then `max:`. A bound writes its unit like any value, in any unit with the same base, converted when compiling, and exactly for an integer type. The same units can add and compare.

```
let $a: i32[unit: time.seconds] = 3600 time.seconds;
let $b: i32[unit: time.seconds] = 5 time.seconds;
let $c: i32[unit: time.seconds] = $a + $b;

use std::units::time.seconds as seconds;
let $d: i32[unit: seconds] = 3600 seconds;
```

A unit is required only where the type asks for one. A plain `i32` means just a number. The language and `std` ask for units on things that have them, like lengths, sizes and durations, and your own code opts in with a type like `i32[unit: time.seconds]`.

```
std::array<u8>[len: 4096 items]
let $x: i32 = 3600;
```

`as` takes a unit off a value, or puts one on: `$xs.len as u64`, `$n as u64[unit: items]`.

A label can carry the unit instead, when the writer declares it that way. A plain number takes the label's unit, and a value in another unit is refused.

```
fn draw [$x: i32[unit: screen.pixels]] -> void:
    labels [$x as at.screen.pixels];
    …
end fn draw

draw(at.screen.pixels: 5);
```

**Refused:** a bound with no unit on a unit type, a bound in another base, an integer bound that doesn't convert exactly, a type's labels out of order, a plain `as` from an integer to a bigger unit, a point plus a point, a value with neither `.point` nor `.span` in a family that needs one, mixing units, as in `$a + 5 storage.bytes`, a missing unit, as in `let $b: i32[unit: time.seconds] = 3600;`, a bare `3600 seconds` with no `use`, and storing a value whose units don't match the declared unit, as in `let $s: f64.stable[unit: motion.speed] = $time div $dist;`.

You define units in a collection. Every unit says what it is: `: base`, or `=` with a conversion, which can point at a unit in any family. Two units convert if they trace back to the same base, and otherwise they don't.

```
fam.unit money:
    money.cents: base;
    money.dollars = 100 money.cents;

    money.euro_cents: base;
    money.euros = 100 money.euro_cents;
end fam.unit money
```

Prefixes are declared once, and a unit opts in with `with`, which creates its prefixed units by joining the names:

```
fam.prefix metric:
    metric.kilo = 1000;
    metric.milli = 1 per 1000;
end fam.prefix metric

fam.unit length:
    length.meters: base with metric;
end fam.unit length

let $road: f64.stable[unit: length.kilometers] = 2.5 length.kilometers;
```

`std` ships `metric` and `binary`, as in `storage.bytes: base with binary;` giving `storage.kibibytes`. `fam.prefix` and `with` are placeholders.

A family is only a group of related names that a generic can ask for. A unit from a kit is written with its path, `name::unit`.

**Refused:** a unit that says neither `base` nor `=`, a loop of `=` lines, and converting between units with different bases.

A unit made of others is declared once, with `per` and `x`, which describe what it's made of rather than calculate anything. The compiler tracks every unit as powers of its bases, so maths can make any combination, but storing a value needs a declared unit whose powers match.

```
fam.unit motion:
    motion.speed = length.meters per time.seconds;
    motion.accel = motion.speed per time.seconds;
    motion.area = length.meters x length.meters;
end fam.unit motion

let $speed: f64.stable[unit: motion.speed] = $dist div $time;
let $accel: f64.stable[unit: motion.accel] = $speed div $time;
```

`std` has an `angle` family, with radians as the base, degrees and turns:

```
use std::units::angle.degrees as degrees;
use std::units::angle.radians as radians;

let $a: f64.stable[unit: degrees] = 90.0 degrees;
let $s: f64.stable = std::math::sin($a as f64.stable[unit: radians]);
```

`std::math::sin` asks for radians, so `std::math::sin(90.0)` is refused, asking for an angle.

A generic can work for any unit and keep it, with a unit parameter. `any.unit` means any unit, and a family's name, like `length`, means any unit in that family:

```
fn average<?u: any.unit> [$xs: std::list<f64.stable[unit: ?u]>[len.min: 1 items]] -> f64.stable[unit: ?u]:
    …
end fn average

let $t: f64.stable[unit: time.seconds] = average($times);
```

A unit parameter goes only where a unit goes, after a number type. A body can add and compare values of the same `?u`.

In a family with an offset, like temperature, and in `std`'s `time`, every value says whether it's a point, a spot on the scale, or a span, the distance between two spots. That covers every unit in the family, its base included, so `kelvin` needs it too:

```
fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius = temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;

let $now: f64.stable[unit: celsius.point] = 20.0 celsius;
let $rise: f64.stable[unit: celsius.span] = 10.0 celsius;
let $later: f64.stable[unit: celsius.point] = $now + $rise;
```

A point plus a span is a point, a point minus a point is a span, and spans add. Converting a point applies the offset, while converting a span only scales, so a rise of 10 °C is a rise of 10 K. `offset` is a placeholder.

A value changes unit with `as`. A float rounds to the nearest, like any float maths. An integer converting to a smaller unit is exact, and to a bigger one says how with a narrowing:

```
let $m: f64.stable[unit: length.meters] = $km as f64.stable[unit: length.meters];
let $k: i32[unit: length.kilometers] = $big as.trunc i32[unit: length.kilometers];
```

`as.trunc`, `as.floor` and `as.exact`, which fails if anything would be lost, follow integer `div` ([3.3](3-operators.md#33-division-div)).

Logarithmic units, like decibels, use the same split. A level is a point and a gain is a span, and the conversion is a log, declared with the placeholder `decibels of`:

```
fam.unit power:
    power.milliwatts: base;
    power.dbm = decibels of 1 power.milliwatts;
end fam.unit power

let $input: f64.stable[unit: power.dbm.point] = 10.0 power.dbm;
let $gain: f64.stable[unit: power.dbm.span] = 3.0 power.dbm;
let $output: f64.stable[unit: power.dbm.point] = $input + $gain;
```

Two levels can't be added, since in dB that would mean multiplying two powers, so you convert to watts, add, and convert back.

**From:** D99, D100, D101, D108, D109, D110, D111, D112, D141, D144, D242, D243, D245, D249, D250, D251, D254, D255, D258, D259, D260.

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

## 2.19 `decl` and `assign`

**Rule:** `decl` declares a variable with no value yet, and `assign` gives it its value, once on each path. Every path must `assign` before any read. After that it acts like a `let`, and `decl mut` lets it change later with `change`.

```
decl $x: i32;
if $hot:
    assign $x = 1;
else:
    assign $x = 2;
end if
std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
```

It isn't uninitialized memory, since nothing can read it empty, so it needs no shield.

**Refused:** reading before every path has assigned it, assigning twice on one path, and `assign` on something that already has a value, with `fix: change`.

**From:** D203.

## 2.20 Types are always written, and Precog's pick: `@precog::set_to_best`

**Rule:** every `let` writes its type. There's no type inference in the base language, so a reader never has to work out a type. With `import @precog;` and its `rule` line on in the manifest, `@precog::set_to_best` leaves the type out, and Precog picks the narrowest plain width that holds every value the variable ever holds:

```
import @precog;

let $count @precog::set_to_best 0;
```

The `@` and the path mark that a limb chose the type. In Precog's mode, a bound it can't prove is a compile error, with no wide fallback, so you write the type there. The same program and target always get the same pick, and the build report lists each one.

**Refused:** a `let` with no type and no Precog, asking for one; `:=`, with `fix: =` and a type; and `@precog::set_to_best` on a parameter, a return type or a struct field.

**From:** D50, D215, D216, D265, D277, D283, and CyborgPL's `:=`.

## 2.21 SIMD

**Rule:** `simd` is a base type holding several values, its lanes, which one CPU instruction works on at once. An operator on two `simd` values works on every lane, and the compiler picks each CPU's instruction, so the same code is portable. In `Literal`, SIMD happens exactly where the code writes `simd` (D282).

```
let $a: simd<i32>[lanes: 4] = [1, 2, 3, 4];
let $b: simd<i32>[lanes: 4] = [10, 20, 30, 40];
let $c: simd<i32>[lanes: 4] = $a + $b;
```

`$c` holds `[11, 22, 33, 44]`. Instructions for one CPU only go in `inline.asm`.

`lanes:` takes any count, and the compiler splits a `simd` into as many instructions as the CPU needs, which never changes the answers, since lanes are independent. Or it takes `max`, the most lanes one instruction works on for the target being built, like 4 `i32`s on the Mac or 1 on the Uno R4 WiFi's Cortex-M4. `max` is fixed for each target, and the build report says what each target got.

```
let $d: simd<i32>[lanes: max] = all 0;
```

A lane follows its type's rules exactly: if any lane overflows or breaks its bound, it's the same failure as for one value of that type, and the message says which lane. `simd<i32[min: 0, max: 100]>` checks every lane's bounds. A check that's proven can be dropped, and the modes treat these checks like any other.

Adding up all the lanes into one value uses a fixed pairwise order: neighbouring lanes are added in pairs, then the results in pairs, until one is left, with an odd lane out carried up to the next round. Four lanes add as `(a + b) + (c + d)`, and three as `(a + b) + c`. So `.stable` and `.ieee` floats give the same answer on every CPU for the same lane count, even when the vector is split, and an integer overflows at the same place every time. `.fast` floats let the compiler pick any order. `lanes: max` changes the lane count between targets, and so the grouping.

Combining the lanes into one value is a `.%` method, since it only reads the `simd`:

```
let $v: simd<i32>[lanes: 4] = [3, 9, 1, 7];
$v.%sum()        // 20
$v.%product()    // 189
$v.%min_val()    // 1
$v.%max_val()    // 9
```

`sum` and `product` use the pairwise order and are checked like any other maths. A `simd<bool>` also has `.%all_true()` and `.%any_true()`.

`as` converts between an array and a `simd` of the same count, both ways. It can't fail, and a different count is refused when compiling.

```
let $a: std::array<i32>[len: 4 items] = [3, 9, 1, 7];
let $v: simd<i32>[lanes: 4] = $a as simd<i32>[lanes: 4];
```

Walking a list or array in `simd`-sized chunks always says how the leftovers are handled, when the length isn't a multiple of the lane count: padding the last chunk with a value you give, requiring an exact multiple, or handing the leftover items over one at a time after the chunks. There's no bare form, since silently dropping the leftovers is a classic SIMD bug.

```
for $chunk in $xs.%simd_chunks.pad(lanes: 4, pad: 0):
    change $total = $total + $chunk;
end for

for $chunk in $ys.%simd_chunks.exact(lanes: 4):
    change $total = $total + $chunk;
end for

for $chunk in $zs.%simd_chunks.leftovers(lanes: 4):
    change $total = $total + $chunk;
leftovers $x:
    change $rest = $rest + $x;
end for
```

With `.leftovers`, the `leftovers` section runs once for each leftover item, after every chunk, and doesn't run when there are none. The loop isn't complete without it. `break` stops the whole loop, leftovers included.

`simd_chunks` only reads: its chunks are copies, and changing one is refused. `.%%simd_chunks.mut` writes each chunk back into the list or array at the end of its round, which is locked for the whole loop. Padding lanes are dropped when writing back, and a changed leftover item is written back too.

```
for $chunk in $pixels.%%simd_chunks.mut.exact(lanes: 16):
    change $chunk = $chunk plus.saturate 20;
end for
```

Comparisons work lane by lane: `==`, `!=`, `below`, `above`, `at_least` and `at_most` on two `simd` values give a `simd<bool>` of the same count. "Every lane" and "any lane" are written out, so the code chooses.

```
let $a: simd<i32>[lanes: 4] = [1, 8, 3, 6];
let $b: simd<i32>[lanes: 4] = [5, 5, 5, 5];
let $m: simd<bool>[lanes: 4] = $a above $b;
if ($a == $b).%all_true():
```

`$m` holds `[false, true, false, true]`.

`.%pick` on a `simd<bool>` chooses each lane from one of two vectors, which is how SIMD code replaces `if`:

```
let $big: simd<i32>[lanes: 4] = $m.%pick(if_true: $a, if_false: $b);
```

`$big` holds `[5, 8, 5, 6]`. Both sides are worked out in full, for every lane, before picking, and every lane is checked, so `$m.%pick(if_true: $x div.trunc $y, if_false: 0)` fails if any lane of `$y` is 0, even one that `if_false` picks.

`.%shuffle` builds a new vector by listing, for each lane of the result, which lane it comes from, with `pos:` counting from 1 or `offset.lanes:` from 0 ([2.17](#217-getting-an-item)). The list is fixed when compiling, since CPUs bake the pattern into the instruction. The result can have a different count, and a lane can repeat.

```
let $v: simd<i32>[lanes: 4] = [10, 20, 30, 40];
let $r: simd<i32>[lanes: 4] = $v.%shuffle(pos: [4, 3, 2, 1]);
let $s: simd<i32>[lanes: 2] = $v.%shuffle(offset.lanes: [0, 0]);
```

`$r` holds `[40, 30, 20, 10]`, and `$s` holds `[10, 10]`.

Four common shuffles have names. `rotate` says its direction, and `by:` is fixed when compiling. `join` and `interleave` take two vectors and give one with both counts added.

```
$v.%reverse()                    // [1, 2, 3, 4] gives [4, 3, 2, 1]
$v.%rotate.left(by: 1)           // [1, 2, 3, 4] gives [2, 3, 4, 1]
$v.%rotate.right(by: 1)          // [1, 2, 3, 4] gives [4, 1, 2, 3]
$a.%join(then: $b)               // [1, 2] and [3, 4] give [1, 2, 3, 4]
$a.%interleave(with: $b)         // [1, 2] and [3, 4] give [1, 3, 2, 4]
```

`.%lookup` takes its lanes from another `simd`, worked out while running, with `pos:` or `offset.lanes:`. A lane pointing past the end is a bounds failure that says which lane, where CPUs quietly give 0 or something else. Bounds on the index type prove every lane in range, so the check can be dropped.

```
let $which: simd<u8[min: 1, max: 16]>[lanes: 16] = …;
let $out: simd<u8>[lanes: 16] = $table.%lookup(pos: $which);
```

**Refused:** a shuffle list only known while running, with `fix: .%lookup`; a lane that doesn't exist, like `pos: [5]` on 4 lanes; a shuffle with neither `pos:` nor `offset.lanes:`; a bare `offset:`, with `fix: offset.lanes:`; changing a chunk from `simd_chunks`; `.%%simd_chunks`, with `fix: .%`, and `.%simd_chunks.mut`, with `fix: .%%`; `if` on a `simd<bool>`, with `fix: .%all_true()` or `.%any_true()`; a bare `simd_chunks`, asking which of the three; `.leftovers` without a `leftovers` section; and a `leftovers` section on any other loop.

**From:** D255, D291, D292, D293, D296, D297, D298, D299, D300, D301, D302, D303, D358, D304, D305, D306, D307.

## 2.22 Secrets: `.secret`

**Rule:** `.secret` on an integer or a `bool` marks a value that mustn't affect how long the program takes, for constant-time code like cryptography. The compiler refuses anything whose timing or path could depend on it. It works inside other types, as in `std::array<u8.secret>[len: 32 items]` or `simd<u8.secret>[lanes: 16]`.

```
let $key: u32.secret = load_key();
let $x: u32.secret = $key xor.bit $nonce;
```

Allowed is what takes the same time for every value on real CPUs: `and.bit`, `or.bit`, `xor.bit` and `not.bit` on secret integers; `and.both`, `or.both` and `not` on a `bool.secret`, since both sides always run; the word forms `plus.wrap`, `minus.wrap`, `times.wrap` and their `.saturate` forms ([3.8](3-operators.md#38-wrapping-and-saturating-pluswrap-plussaturate)); shifts and rotates by a public amount; comparisons, which give a `bool.secret`; and `.%pick(if_true:, if_false:)` on a `bool.secret`, which works out both sides and takes no branch, as on SIMD lanes ([2.21](#221-simd)). A secret mixed with a plain value gives a secret, as in `$key xor.bit 5`, so secrecy spreads to everything it touches and can't be lost by accident.

```
let $same: bool.secret = $a == $b;
let $out: u32.secret = $same.%pick(if_true: $x, if_false: $y);
```

A secret becomes public only with `.%reveal()`, inside a shield ([9.7](9-limbs-and-shields.md#97-shields)), since only the programmer knows it's safe to make public, which is a claim the compiler can't check. The shield's reason says why:

```
shield publish:
    reason "ciphertext is safe to publish once encrypted"
    let $out: u32 = $cipher.%reveal();
end shield publish
```

**Refused:** `.%reveal()` outside a shield; plain `and` and `or` on a `bool.secret`, since stopping early branches on it, with `fix: and.both` or `or.both`; plain `+`, `-` and `x` on a secret, since their overflow check is a branch that leaks it, with `fix: plus.wrap` or `plus.saturate`; bounds on a secret, as in `u8.secret[max: 100]`, unless the check is proven away; `if`, `while` or `match` on a secret, since the path taken leaks it; a secret as a position, as in `$xs[pos: $s]`, since which memory is touched leaks it through the cache; shifting by a secret amount, since some CPUs take longer for bigger shifts; `div` and `mod` with a secret on either side, since division takes longer for some values on most CPUs; and passing a secret where a plain value is expected, printing included.

**From:** D339, D340, D341, D342, D361, D362.
