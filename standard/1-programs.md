# 1. Programs

| Section | What | In short |
|---|---|---|
| [1.1](#11-source-files) | Source files | `.cy` |
| [1.2](#12-main) | `fn.main` | Where a program starts |
| [1.3](#13-blocks-and-statements) | `:`, `;`, `end` | How code is laid out |
| [1.4](#14-names-and-) | `$` | Variables have it, fields don't |
| [1.5](#15-paths-) | `::` | How you reach a kit or limb |
| [1.11](#111-short-names-use) | `use … as …;` | A shorter name for a path |
| [1.12](#112-comments) | `//`, `///`, `comment` | Notes and documentation |
| [1.6](#16-narrowings-) | `.` | Makes the thing before it more specific |
| [1.7](#17-bare-names) | Bare names | The obvious default, or nothing |
| [1.8](#18-brackets) | `( )`, `[ ]`, `< >` | Runs, describes, fills in |
| [1.9](#19-labels-) | `:` in brackets | Names a value, and can be narrowed |
| [1.10](#110-clashing-names-my) | `my::` | A name of yours that clashes |

## 1.1 Source files

**Rule:** a source file ends in `.cy`.

**From:** D16, D137.

## 1.2 `main`

**Rule:** a program starts at `fn.main`, and `main` says `return void;`.

```
import std;

fn.main [] -> void:
    std::println("hi");
    return void;
end fn.main
```

**From:** D11, D23, the sketch.

## 1.3 Blocks and statements

**Rule:** `;` ends a statement, `:` opens a block, and `end` closes it. The closer repeats what it closes, including the name.

```
fn total [$xs: Lent std::list<i32>[len.max: inf items]] -> i32:
    …
end fn total

if $t at_least 10:
    …
end if
```

**From:** D33, D121, the sketch.

## 1.4 Names and `$`

**Rule:** variables, parameters and constants start with `$`. Struct fields don't.

```
let $r: reading = reading[celsius = 21, time = 3600 std::time.seconds];
std::println($r.celsius as str);
```

**From:** D57, D58, the sketch.

## 1.5 Paths: `::`

**Rule:** `::` reaches into a kit or limb by its path. `std` is flat, as in `std::println` and `std::sin`, and any grouping would use `::`, not `.`. A family is one level deep. A file imports what it uses, and a manifest lists what's installed and which version. Everything gets imported, including `std` and limbs, but not builtins or `my::`. A program using only `std` needs no manifest. Imports go at the top of the file. There is no `@import`.

```
import std;
import @csup;

std::println("hi");
```

**From:** D22, D56, D80, D81, D82, D92, D161.

## 1.6 Narrowings: `.`

**Rule:** a `.` makes the thing before it more specific. It also reads a field. It never runs code; a `.%` does ([5.6](5-functions.md#56-methods-)).

```
fn.main
div.floor
status.failed
$r.celsius
```

**Refused:** a narrowing that doesn't mean "this, made more specific" isn't one, so a decimal float is `d32`, not `f32.decimal`.

**From:** D8, D54.

## 1.7 Bare names

**Rule:** a bare name is the obvious default. Where no default is obvious, there is no bare name, and you pick a narrowing.

A default the language or `std` fixes can't be written out. In a collection you write yourself, it can.

```
let $a: str = "hi";
std::println("hi");
7 div.trunc 2
```

**Refused:** `str.utf8`, `std::println.stdout`, a bare `div` on integers, and a bare `mod`.

**From:** D48, D50, D51, D97, D164.

## 1.8 Brackets

**Rule:**

| Bracket | Meaning | Example |
|---|---|---|
| `( )` | Runs something | `total(Lend $xs)` |
| `[ ]` | Describes or holds values | `fn total [$xs: …]`, `i32[min: 0, max: 100]` |
| `< >` | Fills in a type or a source | `std::list<i32>` |

The one exception is grouping in maths: `($a + $b) x 2`.

An opening `<` has no space after it, a closing `>` has none before it, and a comparison has a space on each side.

**Refused:** `$a<$b`.

**From:** D23.

## 1.9 Labels: `:`

**Rule:** a label names a value inside brackets, as in `val: 4` or `len: 8 items`. A label can be narrowed where a type's writer defines that, as in `len.max`. `=` sets a field instead ([7.2](7-structs-and-unions.md#72-building-a-struct)).

```
std::list<i32>[len.min: 1 items, len.max: 8 items]
```

**From:** D13, D57.

## 1.10 Clashing names: `my::`

**Rule:** your own names are bare, and any name is allowed. A name clashes when it's the same as a keyword, a word operator or a base type. The definition stays bare, and every use is written with `my::`.

```
fn x [$a: i32, $b: i32] -> i32:
    …
end fn x

let $area: i32 = $width x $height;
let $mine: i32 = my::x($width, $height);
```

`std::` things don't clash, because they have a path, and variables don't, because they have `$`.

If Cyborg later adds a built-in with the same name, old code gets a compile error saying to write `my::`, never a silent change of meaning.

`my` is a reserved kit prefix. `@my` isn't reserved.

**Refused:** a bare `x(…)` when `x` is built in, with `fix: my::x(…)`.

**From:** D75.

## 1.11 Short names: `use`

**Rule:** `use` gives a path a shorter name for that file, and `as` is always written. An unquoted name has to match the last word of the path, so it can't lie. A quoted name is your own choice, and the quotes only appear on this line.

```
use std::time.seconds as seconds;
use std::print.useful as "printu";

let $a: i32.seconds = 3600 seconds;
printu("hi");
```

A limb works the same way: `use @csup::extern.c as c;`, or `as "ext";`.

A kit method works the same way, with its type in the path and its `%` kept:

```
use std::list.%append as %append;

$xs.%append(val: 4);
```

A method of your own with the same name on the same type in that file clashes, and is refused. A quoted name keeps the `%` outside the quotes, as in `use std::list.%append as %"push";`. A limb's short name has no `@`, since a bare `@name` is a builtin.

**Refused:** `use std::list.%append as "push";` and `as "%push"`, with `fix: as %"push"`, `use std::time.seconds as minutes;`, `use @csup::extern.c as @c;`, because `@c` looks like a builtin, and a quoted name nothing could use, like `"my say"`, `""` or `"fn"`.

**From:** D93, D150, D151, D155, D178, D179.

## 1.12 Comments

**Rule:** `//` is a line comment, a note for whoever reads the code. `///` is a doc comment, attached to what's below it, which tools can show and build documentation from.

```
/// Adds up every item in the list.
fn total [$xs: Lent std::list<i32>[len.max: inf items]] -> i32:
    // a note for whoever reads this
    …
end fn total
```

A comment spanning lines is a block with a required name. Blocks nest, and a closer must name the block it closes. Both lines count only when alone on a line.

```
comment why_sorted:
    The list has to stay sorted because
    find uses a binary search.
end comment why_sorted
```

**Refused:** `//text` with no space, a block with no name, a closer naming the wrong block, and a block that never closes.

**From:** D118, D119.
