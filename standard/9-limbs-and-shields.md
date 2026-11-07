# 9. Limbs and shields

| Section | What | In short |
|---|---|---|
| [9.1](#91-base-kits-and-limbs) | Base, kit, limb | Bare, a path, or `@` |
| [9.2](#92--a-limb) | `@name::` | A limb provides this |
| [9.3](#93-stds-two-parts) | `std::`, `@std::` | The kit part and the limb part |
| [9.4](#94-inline-assembly) | `@std::inline.asm` | In `std`'s limb part |
| [9.5](#95-c-support-csup) | `@csup` | C support |
| [9.6](#96-cs-union) | `@csup::union.untagged` | Inside a shield |
| [9.7](#97-shields) | `shield` | Dangerous code, with a reason |
| [9.8](#98-what-must-live-in-a-shield) | What goes inside | The list |
| [9.9](#99-a-shields-walls) | Entry, exit, containment | The three walls |
| [9.10](#910-precog) | Precog | Proving things |
| [9.11](#911-the-linter) | The linter | An official limb |

Bare `@`, with no `::`, is a compiler builtin instead ([5.7](5-functions.md#57-builtins-)).

## 9.1 Base, kits and limbs

**Rule:**

| Thing | What it is | How it looks |
|---|---|---|
| Base | The language itself | Bare |
| Kit | A library, written in Cyborg | Its path: `std::println` |
| Limb | A compiler extension | `@` and its path: `@csup::union.untagged` |

**From:** D22, D28, D36, D56.

## 9.2 `@`: a limb

**Rule:** `@` with `::` marks something a limb provides, and only that. A search for `@` finds every file using a compiler extension, through its full path or its `use` line, and every builtin. The linter can require the full path. It's a plain `@`, not the fullwidth one Radium uses.

**Refused:** `@std::println`, because `println` is in the kit part.

**From:** D22, D36, D56, D155.

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
        i: i32
        f: f32
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
- Raw pointers
- `@csup` C calls
- `@csup::union.untagged`

Using a limb doesn't need a shield by itself: `@` says it's a compiler extension, and a shield says it's dangerous.

**Refused:** any of these outside a shield, except in `FinalShit`.

**From:** D38, D40.

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

**From:** D14, D29.

## 9.11 The linter

**Rule:** the linter is an official limb like Rust's Clippy. It ships with the compiler, and is configured in the project file. It can force comments, and can show the hidden checks in [2.5](2-values-and-types.md#25-bounded-integers). Linter strictness is separate from the build mode. Definite mistakes stay compiler errors. It can require handling failures that otherwise crash by default ([8.9](8-failures.md#89-crash-by-default)).

**From:** D14, D21, D30, D96.
