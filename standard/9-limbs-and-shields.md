# 9. Limbs and shields

| Section | What | In short |
|---|---|---|
| [9.1](#91-base-kits-and-limbs) | Base, kit, limb, package | Bare, a path, or `@` |
| [9.2](#92--a-limb) | `@name::` | A limb provides this |
| [9.3](#93-stds-two-parts) | `std::`, `@std::` | The kit part and the limb part |
| [9.4](#94-inline-assembly) | `@std::inline.asm` | In `std`'s limb part |
| [9.5](#95-c-support-csup) | `@csup` | C support |
| [9.6](#96-cs-union) | `@csup::union.untagged` | Inside a shield |
| [9.7](#97-shields) | `shield` | Dangerous code, with a reason |
| [9.8](#98-what-must-live-in-a-shield) | What goes inside | The list |
| [9.9](#99-a-shields-walls) | Entry, exit, containment | The three walls |
| [9.10](#910-precog) | Precog | Proving things |
| [9.11](#911-the-linter) | The linter | Part of the compiler |
| [9.12](#912-pub) | `pub` | Seen outside its package |
| [9.13](#913-a-files-rules-config) | `@config` | The rules a file is held to |

Bare `@`, with no `::`, is a compiler builtin instead ([5.7](5-functions.md#57-builtins-)).

## 9.1 Base, kits and limbs

**Rule:**

| Thing | What it is | How it looks |
|---|---|---|
| Base | The language itself | Bare |
| Kit | A library, written in Cyborg | Its path: `std::io::fmt_print` |
| Limb | A compiler extension | `@` and its path: `@csup::union.untagged` |
| Package | What you install: a kit, a limb, or both, with one name and one version | Its halves, as `std::` and `@std::` |

The manifest lists packages. `std` is a package with both halves. Your own project is a package too, named in the manifest, and no file says which package it belongs to. The base and builtins aren't packages.

Packages are installed with CyJet, Cyborg's package manager, which shares only its name with CyborgPL's.

**Refused:** a line like Go's `package main`.

**From:** D22, D28, D36, D56, D186, D189, D190.

## 9.2 `@`: a limb

**Rule:** `@` with `::` marks something a limb provides, and only that. A search for `@` finds every file using a compiler extension, through its full path, its `use` line or its import, and every builtin, and a search for `pkg` finds the rest, since `import pkg std;` brings in a limb half without an `@`. The linter can require the full path. It's a plain `@`, not the fullwidth one Radium uses.

**Refused:** `@std::io::fmt_print`, because `fmt_print` is in the kit part.

**From:** D22, D36, D56, D155, D188.

## 9.3 `std`'s two parts

**Rule:** bare `std::` reaches the kit part, written in Cyborg and built by the compiler. `@std::` reaches the limb part, which lives in the compiler, in Rust, with its names and signatures visible in Cyborg.

**From:** D22, D28.

## 9.4 Inline assembly

**Rule:** `@std::inline.asm` is in `std`'s limb part, and must be inside a shield.

**From:** D28, D35, D36, D38.

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
        f: f32;
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

- `@std::inline.asm`
- Raw pointers, with any address and pointer arithmetic
- Volatile reads and writes, for hardware registers
- Exact struct layouts, with field order, packing and alignment fixed
- Your own allocator
- `@csup` C calls
- `@csup::union.untagged`

So inside a shield, Cyborg goes as low as C, and a bare-metal program can run with no allocator at all. How the new ones are spelled isn't decided.

Using a limb doesn't need a shield by itself: `@` says it's a compiler extension, and a shield says it's dangerous.

**Refused:** any of these outside a shield, except in `FinalShit`.

**From:** D38, D40, D226.

## 9.9 A shield's walls

**Rule:**

| Wall | What it does | How it's enforced |
|---|---|---|
| Entry | The caller meets `requires` | At runtime unless Precog proves it, following the build mode |
| Exit | `ensures` holds | [10.3](10-build-modes.md#103-what-gets-checked) |
| Containment | Raw pointers and unchecked borrows can't leave | A compile error, except in `FinalShit` |

**From:** D37, D39.

## 9.10 Precog

**Rule:** Precog proves things about a program. Layers 1 and 2, known values and ranges, are in the core. Layer 3, Z3 and CVC5, is a limb. That limb adds invariants, `assert` hints, ghost code, lemmas, proofs that loops end and that stack and heap fit their budgets, proof levels, and a "prove everything or refuse" mode as an option. Anything Precog proves can drop its check safely, in any mode.

With its rule on, the limb also picks the type for `:=` ([2.20](2-values-and-types.md#220--inference-or-precogs-pick)).

**From:** D14, D29.

## 9.11 The linter

**Rule:** the linter is part of the compiler, not a limb, and its rules are set with a file's rules ([9.13](#913-a-files-rules-config)). A limb can add rules of its own, named by its path, as in `@house_style::fn::name_length`. It can force comments, and can show the hidden checks in [2.5](2-values-and-types.md#25-bounded-integers). Linter strictness is separate from the build mode. Definite mistakes stay compiler errors. It can require handling failures that otherwise crash by default ([8.9](8-failures.md#89-crash-by-default)).

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

## 9.13 A file's rules: `@config`

**Rule:** every file's first line of code is `@config(…);`, with only comments allowed above it, which sets its first rules, or `@match_config(that file);`, which copies another file's. The compiler's defaults are `@config(default);`, since there's no unsaid default. Any `@tighten_config(…);`, which only makes rules stricter, or `@loosen_config(…);`, which only makes them looser, comes straight after it, before any `import` or `use`.

```
@config(default);
@tighten_config(var::mut::never_changed = refuse);

fn.main [] -> void:
    let mut $x: i32 = 1000;
    return void;
end fn.main
```

The `let mut` is refused, since `$x` never changes. The spelling of a rule is a sketch.

A search for `@loosen_config` finds every file that weakens its rules. The manifest only handles packages.

**Refused:** a file with neither `@config` nor `@match_config`, either one anywhere but the first line, a tighten or loosen line after anything else, both in one file, loosening with `@tighten_config` or tightening with `@loosen_config`, a cycle of `@match_config`, and a matched file that's missing.

**From:** D30, D56, D201, D204, D205.
