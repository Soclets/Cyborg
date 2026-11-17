# Design claude's bet on Claude's own language

Written on 2026-11-17 at 20:23, before the session from `own-language-prompt.md` started, so it can be checked
afterwards. The owner asked for the bet; nothing here steered that session.

## Philosophy

- Memory safety without a garbage collector, but simpler than Rust: most likely references that can't be stored
  ("second-class" references) or mutable value semantics, as in Hylo, to avoid lifetime annotations.
- Zig-style explicitness: no hidden control flow, explicit allocators, `defer`, and compile-time execution in place of
  macros.
- Errors as values, with a `?`-like operator, and no exceptions.
- C interop and fast compile times as stated goals.
- A tagline along the lines of "predictable" or "no surprises".
- What it won't have: a joke build mode with a curse, three kinds of shields, or `~0.1`-style inexact literals.

## Name

Short, one syllable, a craft or nature word, like Keel, Forge, Tern, Slate or Ore. Single pick: **Keel**. Runner-up:
**Forge**.

## Backend

- The compiler written in Rust.
- LLVM for optimized builds, through its C API or a crate like `inkwell`, since it's the standard answer for a
  systems language that wants C-level speed and many targets.
- A second, fast backend for debug builds, most likely Cranelift, justified by compile speed, possibly stated as a
  later stage rather than built first.
- Not its own code generator, and not compiling to C, though compiling to C may be weighed and rejected in its
  decision log.
