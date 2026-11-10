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
| [1.13](#113-printing-stdiofmt_print) | `std::io::fmt_print` | The one print, a format print |
| [1.14](#114-spacing) | Spaces | Part of the syntax |
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
use std::io::fmt_print as fmt_print;

fn.main [] -> void:
    fmt_print("Hello, World!{newln}");
    return void;
end fn.main
```

This is the owner's hello world. The `use` line is on purpose.

**From:** D11, D23, D93, D197, D265, the sketch.

## 1.3 Blocks and statements

**Rule:** `;` ends a statement, `:` opens a block, and `end` closes it. The closer repeats what it closes, including the name.

```
fn total [$xs: std::list<i32>[len.max: inf items]] -> i32:
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
let $r: reading = reading[celsius = 21, time = 3600 std::units::time.seconds];
std::io::fmt_print("{}{newln}", $r.celsius);
```

**From:** D57, D58, the sketch.

## 1.5 Paths: `::`

**Rule:** `::` reaches into a kit or limb by its path. `std` is grouped with `::`, as in `std::math::sin`, and `use` shortens a long path. `::` is for paths, which never run anything. A family is one level deep. A file imports what it uses, and a manifest lists which packages are installed and which version ([9.1](9-limbs-and-shields.md#91-base-kits-and-limbs)). Everything gets imported, including `std` and limbs, but not builtins or `my::`. Every program has a manifest, `CyJet.siml`, even one using only `std`. Imports go at the top of the file. There is no `@import`.

```
import std;
import @csup;

std::io::fmt_print("hi{newln}");
```

A file imports the half of a package it uses: `import std;` for `std::`, and `import @std;` for `@std::`. `import pkg std;` brings in both.

`import std::io::fmt_print;` imports only that one thing, with its narrowings, like `.stderr`, and the file can reach nothing else from `std`. It still writes the full path, unless a `use` shortens it. `import std::units::time;` imports a whole unit family.

**From:** D22, D56, D80, D81, D82, D92, D161, D183, D186, D187, D188, D191, D196, D198, D199, D229.

## 1.6 Narrowings: `.`

**Rule:** a `.` makes the thing before it more specific. It also reads a field. It never runs code; a `.%` does ([5.6](5-functions.md#56-methods-)).

```
fn.main
div.floor
status.failed
$r.celsius
```

**Refused:** a narrowing that doesn't mean "this, made more specific" isn't one, so a decimal float is `d32`, not `f32.decimal`.

With two or more narrowings, the strongest goes first: the one that changes the action most, or for safety words, the one that can do more harm. The order is fixed. Kinds so far, strongest first:

| Thing | Order | Example |
|---|---|---|
| A print | What's printed, then where | `fmt_print.stored.stderr` |
| A borrow | How it can be used, then how long | `Lend.mut.held` |

Shapes in a format hole are labels, so they go in any order.

**Refused:** narrowings out of order, as in `fmt_print.stderr.stored`, with the right order as the fix.

**From:** D8, D54, D223.

## 1.7 Bare names

**Rule:** a bare name is the obvious default. Where no default is obvious, there is no bare name, and you pick a narrowing.

A default the language or `std` fixes can't be written out. In a collection you write yourself, it can.

```
let $a: str = "hi";
std::io::fmt_print("hi{newln}");
7 div.trunc 2
```

**Refused:** `str.utf8`, `std::io::fmt_print.stdout`, a bare `div` on integers, and a bare `mod`.

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

**Rule:** your own names are bare, and any name is allowed. A name clashes when it's the same as a keyword, a word operator, a base type, or `main`. It's written with `my::` everywhere: its declaration, its closer and every use.

```
fn my::x [$a: i32, $b: i32] -> i32:
    …
end fn my::x

let $area: i32 = $width x $height;
let $mine: i32 = my::x($width, $height);
```

`std::` things don't clash, because they have a path, and variables don't, because they have `$`.

If Cyborg later adds a built-in with the same name, old code gets a compile error saying to write `my::`, never a silent change of meaning.

`my` is a reserved kit prefix. `@my` isn't reserved.

**Refused:** a bare `x(…)` when `x` is built in, with `fix: my::x(…)`, a bare `fn x` declaring one, with `fix: fn my::x`, and `fn main`, with `fix: fn.main`, since a plain function named `main` would sit one character away from the program's start. A function really named `main` is `fn my::main`.

**From:** D75, D214.

## 1.11 Short names: `use`

**Rule:** `use` gives a path into a kit or limb a shorter name for that file, and `as` is always written. Anything in the base language, like `inline.asm`, has no path to shorten, so `use` refuses it. Your own package's path is `my::`, so `use my::temperature.celsius as celsius;` shortens a unit, type or function you declared. An unquoted name has to match the last word of the path, so it can't lie. A quoted name is your own choice, and the quotes only appear on this line.

```
use std::units::time.seconds as seconds;
use std::io::fmt_print as "say";

let $a: i32[unit: seconds] = 3600 seconds;
say("hi{newln}");
```

A limb works the same way: `use @csup::union.untagged as untagged;`, or a quoted name.

A kit method works the same way, with its type in the path and its `%` kept:

```
use std::list.%%append as %%append;

$xs.%%append(val: 4);
```

A method of your own with the same name on the same type in that file clashes, and is refused. A quoted name keeps the `%` or `%%` outside the quotes, as in `use std::list.%%append as %%"push";`. A limb's short name has no `@`, since a bare `@name` is a builtin.

A function's narrowing can't be shortened to its bare name, since `stored(…)` alone reads like something else. Shorten the function, and write the narrowing after it, or use a quoted name:

```
use std::io::fmt_print as fmt_print;
use std::io::fmt_print.stored as "print_exact";

fmt_print.stored("{}{newln}", $x);
print_exact("{}{newln}", $x);
```

Units keep the matching name, as in `use std::units::time.seconds as seconds;`.

**Refused:** `use std::io::fmt_print.stored as stored;`, with `fix: use std::io::fmt_print as fmt_print;`, `use std::list.%%append as "push";` and `as "%%push"`, with `fix: as %%"push"`, `as %append`, which drops a `%`, `use std::units::time.seconds as minutes;`, `use @csup::union.untagged as @untagged;`, because `@untagged` looks like a builtin, and a quoted name nothing could use, like `"my say"`, `""` or `"fn"`.

**From:** D93, D150, D151, D155, D178, D179, D220, D225, D232, D248.

## 1.12 Comments

**Rule:** comments can go anywhere. `//` is a line comment, a note for whoever reads the code, and it can trail code on the same line, as in `let $x: i32 = 1; // a note`. `///` is a doc comment, attached to what's below it, which tools can show and build documentation from.

```
/// Adds up every item in the list.
fn total [$xs: std::list<i32>[len.max: inf items]] -> i32:
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

**Refused:** `//text` with no space, a `///` after code on the same line, with a fix that moves it above, a block with no name, a closer naming the wrong block, and a block that never closes.

**From:** D118, D119, D205, D206.

## 1.13 Printing: `std::io::fmt_print`

**Rule:** `std` has one print, `std::io::fmt_print`, and it's always a format print. Plain text is a format with no holes. A newline is `{newln}` and a tab is `{tab}`, inside the text.

```
std::io::fmt_print("hi{newln}");
std::io::fmt_print("{}: {} at ${places: 2}{newln}", $name, $count, $price);
std::io::fmt_print("0x{base: hex.upper, zeros, width: 4 cols}{newln}", 255);
std::io::fmt_print("[{align: right, width: 6 cols}] {form: science, places: 2}{newln}", 42, $price);
std::io::fmt_print.stored("{}{newln}", $tenth);
```

With `$name` as `"rope"`, `$count` as `3`, `$price` as an `f64.stable` holding `12.5`, and `$tenth` as an `f64.stable` holding `~0.1`, these print `hi`, `rope: 3 at $12.50`, `0x00FF`, `[    42] 1.25e1`, and `0.1000000000000000055511151231257827021181583404541015625`.

- **Holes:** `{}` takes the next value in its readable form. `{1}`, `{2}` are numbered, counting from 1, and can't be mixed with plain ones.
- **Named holes:** `{$t}` prints the variable `$t`, with shapes after a comma and fields allowed, as in `fmt_print("{$name}: {$count} at ${$price, places: 2}{newln}");` and `{$r.celsius}`. A hole only reads a value, never a call or maths. Named holes aren't mixed with `{}` or numbered ones.
- **Shapes are words:** `align: left`, `right` or `center`; `width:` with its unit, `cols`, `cols.east_asian`, `graphemes`, `points` or `bytes`; `places:`; `base: bin`, `oct`, `hex` or `hex.upper`; `form: science`; and bare `zeros` and `sign`.
- **Escapes in the text:** `\\`, `\"`, `\'`, `\r`, `\0`, `\xHH`, `\uHHHH`, `\UHHHHHHHH`, and `\{` and `\}` for a brace.
- **Too wide:** a value wider than its width prints whole.
- **`unit`:** prints the value and its unit, the way its own file writes it, so with `use std::units::time.seconds as seconds;`, `{unit}` prints `3600 seconds`. The package path is left off.
- **`.stored`:** prints every float exactly as it's stored.
- **`.stderr`:** prints to stderr instead of stdout, as in `std::io::fmt_print.stderr("failed{newln}");`.
- **Both:** `.stored` and `.stderr` combine on one print, strongest first ([1.6](#16-narrowings-)), as in `std::io::fmt_print.stored.stderr(…)`.

A literal format is checked when building.

**Refused:** a bare `{` in the text, with `fix: \{`; `\n`, with `fix: {newln}`; `{0}`; mixing plain and numbered holes; more or fewer values than holes; a shape that doesn't apply, like `places:` on an integer; a width with no unit; `zeros` with no width; a shape written twice; an unknown escape; `std::io::fmt_print.stdout`, since stdout is the default, and `std::io::fmt_println`, with `fix: std::io::fmt_print`.

**From:** D11, D50, D51, D122, D162, D165, D194, D195, D197, D210, D221, D223, D246, D247, and CyborgPL's format print.

## 1.14 Spacing

**Rule:** spacing is part of the syntax, so every program reads the same.

| Where | Spaces | Example |
|---|---|---|
| Around `.`, `::`, `%`, `%%`, `@` | None | `fn.main`, `std::io::fmt_print`, `$p.%area()` |
| After `$`, a unary `-`, a `~` on a literal, an opening `(`, `[`, `<` | None | `-$x`, `~0.1`, `total(Lend $xs)`, `std::list<i32>` |
| Before a closing bracket, or a postfix `?` | None | `read()?` |
| Around a binary operator, word operators and `=` included | One on each side | `1 < 8`, `$a + $b`, `$x at_least 0` |
| After a comma, or a label's `:` | One | `range[from: 1, to: 3]` |

**Refused:** `fn . main`, `std :: io`, `1<8`, `$a+$b`, `$a  +  $b`, and `( $x )`, each with the spaced version as its fix, and a line break anywhere else, such as between `end` and `fn.main`, with a fix that joins the line.

A line break inside a statement is allowed only after a comma, after an opening `(`, `[` or `<`, with the closing one then on its own line, and after a binary operator. Every block opener and closer stays on one line.

```
std::io::fmt_print(
    "{}: {} at ${places: 2}{newln}",
    $name,
    $count,
    $price
);
```

Indentation, line length, and the space before an opening `[` aren't decided ([12.1](12-not-decided.md#121-problems-found-writing-this)).

**From:** D23, D213, D219.
