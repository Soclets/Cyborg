# 9. Limbs and shields

| Section | What | In short |
|---|---|---|
| [9.1](#91-base-kits-and-limbs) | Base, kit, limb, package | Bare, a path, or `@` |
| [9.2](#92--a-limb) | `@name::` | A limb provides this |
| [9.3](#93-stds-two-parts) | `std::`, `@std::` | The kit part and the limb part |
| [9.4](#94-inline-code-inline) | `inline.asm`, `inline.@limb::lang` | In the base language, and limbs add to it |
| [9.5](#95-c-support-csup) | `@csup` | C support |
| [9.6](#96-cs-union) | `@csup::union.untagged` | Inside a shield |
| [9.7](#97-shields) | `shield` | Dangerous code, with a reason |
| [9.8](#98-what-must-live-in-a-shield) | What goes inside | The list |
| [9.9](#99-a-shields-walls) | Entry, exit, containment | The three walls |
| [9.10](#910-precog) | Precog | Proving things |
| [9.11](#911-the-linter) | The linter | Part of the compiler |
| [9.12](#912-pub) | `pub` | Seen outside its package |
| [9.13](#913-rules-rule) | `rule` | The rules code is held to, in the manifest |

Bare `@`, with no `::`, is a compiler builtin instead ([5.7](5-functions.md#57-builtins-)).

## 9.1 Base, kits and limbs

**Rule:**

| Thing | What it is | How it looks |
|---|---|---|
| Base | The language itself | Bare |
| Kit | A library, written in Cyborg | Its path: `std::io::fmt_print` |
| Limb | A compiler extension | `@` and its path: `@csup::union.untagged` |
| Package | What you install: a kit, a limb, or both, with one name and one version | Its halves, as `std::` and `@std::` |
| Dialect | A compiler forked from `OKAY\n`, on CyJet's registry, with no rules of Cyborg's | A `dialect` line in the manifest |

The manifest lists packages. `std` is a package with both halves, and works anywhere, bare metal included: collections, maths, units, text and formatting. `os` is a separate package, a plain and portable layer over an operating system, like the terminal, files, arguments and exit codes, so a program using it says `need os`. Your own project is a package too, named in the manifest, and no file says which package it belongs to. The base and builtins aren't packages.

Packages are installed with CyJet, Cyborg's package manager, which shares only its name with CyborgPL's.

The manifest, `CyJet.siml`, has one line per fact:

```
name weather
ver 2026y-11m-14d same
dialect original 2026y-11m-01d

need std 2026y-11m-01d
need os 2026y-11m-01d
need json 2026y-10m-20d
need csup 2026y-09m-30d with headers
```

`name` and `ver` are the project's own. A version is a labelled date, biggest first, and a release says whether it `breaks` anything or stays the `same`, a placeholder word. CyJet checks that flag against what changed in the `pub` items. A `need` line names a date, and CyJet may update it to any later release up to the next one that breaks. A second release on one day adds a counter, as in `2026y-11m-14d.2`. `dialect` says which compiler builds the project, and which version of it: `dialect original 2026y-11m-01d` is plain Cyborg, built by `okay` at that version, and any other name is a Dialect, a compiler forked from `OKAY\n` that CyJet loads from its registry, and which needn't follow any of Cyborg's rules. Every manifest has a `dialect` line, and `original` is reserved, so no Dialect can take it. A `dialect original` project can't use a package written in a Dialect, and CyJet stops it when installing, since that code needs another compiler and none of Cyborg's checks cover it. Whether a Dialect project can use plain Cyborg packages is up to the Dialect. Each package it uses has a `need` line with its version, and `with` adds features. A value is quoted only when it needs to be, and `//` starts a comment, as in Cyborg code.

**Refused:** a manifest with no `dialect` line, with `fix: dialect original`; an `okay` line, with `fix: dialect original` and its version; a Dialect named `original`; a `dialect original` project needing a package written in a Dialect; a line like Go's `package main`, a `need` line with no version, and a release that changes or removes a `pub` item while saying it doesn't break.

**From:** D22, D28, D36, D56, D186, D189, D190, D228, D229, D230, D231, D271, D309, D310, D311.

## 9.2 `@`: a limb

**Rule:** `@` with `::` marks something a limb provides, and only that. A search for `@` finds every file using a compiler extension, through its full path, its `use` line or its import, and every builtin, and a search for `pkg` finds the rest, since `import pkg std;` brings in a limb half without an `@`. The linter can require the full path. It's a plain `@`, not the fullwidth one Radium uses.

**Refused:** `@std::io::fmt_print`, because `fmt_print` is in the kit part.

**From:** D22, D36, D56, D155, D188.

## 9.3 `std`'s two parts

**Rule:** bare `std::` reaches the kit part, written in Cyborg and built by the compiler. `@std::` reaches the limb part, a limb like any other.

Every limb is a native library for the machine compiling, a `.dylib`, `.so` or `.dll`, behind a stable C interface, loaded by `OKAY\n` while compiling. `std`'s limb half and Precog plug in the same way as anyone else's, with none privileged. A limb isn't sandboxed: it's trusted when the manifest lists it, and an unverified one gets a warning, not a refusal: "a limb runs inside the compiler and can change what your program does. Use it only if you trust where it came from." A limb runs only while compiling, so a program is the same whether its limbs were built in or loaded. A limb could make builds differ, by reading a clock or a file, and keeping them reproducible is its author's job.

A limb's syntax is only ever `@`-marked items, in a fixed menu of positions, so it never reserves a plain word and two limbs never clash:

| Position | Example |
|---|---|
| A type | `@csup::union.untagged` |
| A call or value | `@money::round_half_even($x)` |
| A statement | `@trace::here;` |
| A block with Cyborg inside | `@gpu::kernel …: … end @gpu::kernel` |
| A raw block in another language | `inline.@llvm::ir: … end inline.@llvm::ir` |
| In place of `=` in a `let` | `let $count @precog::set_to_best 0;` |
| A rule in the manifest | `rule @house_style::fn::name_length` |

Raw text that Cyborg doesn't parse is only allowed inside an `inline.` block, so every file stays readable by any Cyborg reader, editor and tool. The names in the examples are made up.

**From:** D22, D28, D334, D335.

## 9.4 Inline code: `inline`

**Rule:** `inline` is a family in the base language. Inline assembly is `inline.asm`, written bare, with no import. A limb adds its own inline language to the family as `inline.@limb::lang`, so the `@` still marks it. All inline code must be inside a shield.

```
inline.asm
inline.@llvm::ir
```

This is the one place a path sits inside a narrowing, so every limb plugs into the same `inline` instead of declaring its own.

An assembly block says in words what goes in and out, then holds raw assembly, one instruction per line, which LLVM's assembler reads. It's a statement, and must be inside a shield and inside an `on.target` choice naming the chip:

```
on.target aarch64:
    shield mul_add:
        reason "one madd instruction computes a times b plus c"
        inline.asm:
            reads: [x0 = $a, x1 = $b, x2 = $c];
            writes: [x0 => $r];
            clobbers: [none];
            madd x0, x0, x1, x2
        end inline.asm
    end shield mul_add
else:
    let $r: i64 = $a x $b + $c;
end on.target
```

- **`reads:`** puts values into registers before it runs.
- **`writes:`** takes registers into variables after, declaring them as a `let` would.
- **`clobbers:`** lists every other register the code changes.
- **An empty list** in any of the three is `[none]`, as in `clobbers: [none];`.
- **`on.target`** compiles a branch only when building for that chip, with `else:` for the rest.

The compiler reads the text through LLVM and refuses any register the code writes that isn't in `writes:` or `clobbers:`. A value narrower than its register needs an `as`.

**Refused:** an `inline.asm` block outside an `on.target` choice or outside a shield, a register written but not declared, a narrow value with no `as`, `@std::inline.asm`, with `fix: inline.asm`, `inline.llvm::ir`, with no `@`, and any `use` of an `inline.…`, since `use` doesn't work on the base language.

**From:** D28, D35, D36, D38, D227, D232, D233, D269, and CyborgPL's `on.target`.

## 9.5 C support: `@csup`

**Rule:** the limb `@csup` holds C: headers read directly, C's layouts, calls through C's conventions, and generated `.h` files so C and C++ can call into Cyborg by name. There are walls at the boundary. There's no C++ support for now, and a `@cppsup` limb could come later. The base language has no C in it, and a project lists the limb when it needs it.

**From:** D31, D35, D156.

## 9.6 C's union

**Rule:** C's untagged union has its own spelling, and must be inside a shield. It can't be tested, because nothing records which case is live.

```
shield c_number:
    reason "matches the union in libfoo's header"
    @csup::union.untagged number:
        i: i32;
        f: f32.ieee;
    end @csup::union.untagged number
end shield c_number
```

**From:** D33, D35, D38.

## 9.7 Shields

**Rule:** a shield is a named block with a written reason, and it's the only place dangerous code may live. The build report lists each one with its size. Cyborg isn't all proven, so a shield is closer to Rust's `unsafe`, unless the Precog limb's strict mode is on.

The keyword `shield` is Radium's working word.

```
shield sensor_driver:
    reason "talks to the temperature chip over raw memory-mapped I/O"
    …
end shield sensor_driver
```

**From:** D34.

## 9.8 What must live in a shield

**Rule:**

- `inline.asm`, and any `inline.@limb::lang`
- Raw pointers, with any address and pointer arithmetic
- Volatile reads and writes, for hardware registers
- Your own allocator
- `@csup` C calls
- `@csup::union.untagged`

So inside a shield, Cyborg goes as low as C, and a bare-metal program can run with no allocator at all.

A raw pointer is `ptr.raw<T>`, which can only read, or `ptr.raw.mut<T>`, which can read and write. `ptr` is a family, so other kinds of pointer can join it as narrowings.

```
let $reg: ptr.raw.mut<u32> = ptr.raw.mut<u32>.at(address: 0x4004_0000);
let $p: ptr.raw.mut<i32> = ptr.raw.mut<i32>.of(Lend.mut $count);
let $now: u32 = $reg.%read();
$reg.%%write(val: 1);
```

`.at(address:)` makes a pointer from a number, for hardware registers and memory maps. `.of` takes a variable's address, with `Lend` for `ptr.raw` and `Lend.mut` for `ptr.raw.mut`, and the variable stays lent for as long as the pointer exists, while containment keeps the pointer in the shield ([9.9](#99-a-shields-walls)). `.%read()` reads, and `.%%write(val:)` writes, where `%%` marks the change to what the pointer stands for.

`.%moved` gives a new pointer, always saying its unit, so one item and one byte can't be confused, and a negative number moves backwards. Nothing is checked, and nothing happens until a read or a write.

```
let $next: ptr.raw.mut<u32> = $reg.%moved(offset.items: 1);
let $odd: ptr.raw.mut<u32> = $reg.%moved(offset.bytes: 2);
```

A volatile pointer, `ptr.volatile<T>` or `ptr.volatile.mut<T>`, is for hardware registers: every read and write through it happens, exactly once and in order, and the compiler never skips, merges or reorders them. It works like `ptr.raw`, with `.at`, `.of`, `.%read()`, `.%%write(val:)` and `.%moved`.

```
let $led: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: 0x4004_0000);
$led.%%write(val: 1);
$led.%%write(val: 0);
```

Writing your own allocator is shield code. How an allocator is spelled and what it must provide isn't decided ([2.10](2-values-and-types.md#210-lists)).

Using a limb doesn't need a shield by itself: `@` says it's a compiler extension, and a shield says it's dangerous.

**Refused:** any of these outside a shield, except in `FinalShit`; writing through a `ptr.raw` or a `ptr.volatile`; and `.%moved` with a bare `offset:`, with `fix: offset.items:`.

**From:** D38, D40, D226, D314, D315, D316, D317.

## 9.9 A shield's walls

**Rule:**

| Wall | What it does | How it's enforced |
|---|---|---|
| Entry | The caller meets `requires` | At runtime unless it's proven, by `OKAY\n` or Precog, following the build mode |
| Exit | `ensures` holds | [10.3](10-build-modes.md#103-what-gets-checked) |
| Containment | Raw pointers and unchecked borrows can't leave | A compile error, except in `FinalShit` |

**From:** D37, D39.

## 9.10 Precog

**Rule:** Precog is a limb, `@precog`, and only the limb has the name. Without it, `OKAY\n` still works out known values and ranges itself, and drops checks that can't fail, as its own optimization, the way it folds constants. Precog adds Z3 and CVC5, and with them invariants, `assert` hints, ghost code, lemmas, proofs that loops end and that stack and heap fit their budgets, proof levels, and a "prove everything or refuse" mode as an option. Anything `OKAY\n` or Precog proves can drop its check, in every mode except `Working`, which keeps them all.

With its rule on, the limb also picks a variable's type, with `@precog::set_to_best` ([2.20](2-values-and-types.md#220-types-are-always-written-and-precogs-pick-precogset_to_best)).

**From:** D14, D29, D312.

## 9.11 The linter

**Rule:** the linter is part of the compiler, not a limb, and its rules are set with `rule` lines in the manifest ([9.13](#913-rules-rule)). A limb can add rules of its own, named by its path, as in `@house_style::fn::name_length`. It can force comments, and can show the hidden checks in [2.5](2-values-and-types.md#25-bounded-integers). Linter strictness is separate from the build mode. Definite mistakes stay compiler errors. It can require handling failures that otherwise crash by default ([8.9](8-failures.md#89-crash-by-default)).

**From:** D14, D21, D30, D96, D201, D202.

## 9.12 `pub`

**Rule:** anything a package declares is private to that package unless it starts with `pub`. `pub` is a separate word in front, not a narrowing, and it works on anything a package declares.

```
pub fn total [$xs: std::list<i32>[len.max: inf items]] -> i32:
pub fn.pure area [$w: i32, $h: i32] -> i32:
pub struct reading:
pub const $max: i32 = 100;
```

`fn.main` needs no `pub`.

**Refused:** `fn.pub`, with `fix: pub fn`.

**From:** D186, D200.

## 9.13 Rules: `rule`

**Rule:** the rules the compiler and its linter hold code to are `rule` lines in `CyJet.siml`. They apply to every file in the project, and with no `rule` lines, the compiler's defaults apply. An exception for one file names the file:

```
name weather
ver 2026y-11m-14d same
dialect original 2026y-11m-01d

need std 2026y-11m-01d

rule var.mut.never_changed refuse
rule var.mut.never_changed allow in drivers/raw.cy
```

So a `let mut` that never changes is refused everywhere except `drivers/raw.cy`. Every exception is listed in the manifest. `rule`, `refuse`, `allow` and `in` are placeholders, and how a rule's name is spelled isn't decided.

Code has no rules lines of its own.

**Refused:** `@config`, `@match_config`, `@tighten_config` and `@loosen_config`.

**From:** D30, D56, D201, D202, D204, D265.
