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

## Tankun's bet

Written on 2026-11-20 at 16:29, after the stage 1 prompt asked for at least one major feature as a reason to adopt the
language, and before any session ran it. In Tankun's words: "General purpose sys lang, no hidden control flow & alloc,
light Rust, but with compile checked units".

- A general-purpose systems language.
- No hidden control flow and no hidden allocation.
- "Light Rust": Rust's safety with simpler ownership.
- The major feature: units of measure checked while compiling, which F# has but no mainstream systems language does.

## Design claude's second bet

Written on 2026-11-20 at 16:32, after the stage 1 prompt asked for a major feature and after Tankun's bet, before any
session ran it.

- The major feature: an effect system for systems code. Every function's header says what it can do, such as
  allocate, do I/O, panic, block or touch unchecked memory, and the compiler checks it all the way up the call chain,
  so a function marked as doing none of these provably can't, and the language's "no hidden control flow and
  allocation" becomes something the compiler proves rather than a promise. Likely carried by capabilities, values
  passed in like Zig's allocators but for every effect. Ideas from Koka and Austral, framed for systems work.
- Runner-up for the major feature: contracts or refinement types proven while compiling, as SPARK does.
- The rest of the first bet still stands: memory safety without a garbage collector, simpler than Rust, errors as
  values, a Rust compiler on LLVM, and a short name, with Keel still the pick.

## Result of stage 1

On 2026-11-20 at 18:08, the session ("Systems programming language design", in `/Users/ts/Folder titled untitled`)
reported it was done, by message, without design claude opening its folder:

- The language is **Tally**: 23 chapters, 519 rules, 69 decisions, and an open-questions chapter, reviewed three times.
- Its headline feature is "cost accounting": effects (`alloc`, `block`, `panic`, `sys`) written in every public
  signature and checked through every call, so a `may none` function provably can't allocate, wait, crash or make
  system calls. The linker works out worst-case stack sizes from the real machine code, so fibers get exactly-sized
  stacks and there's no `async`/`await` split.
- Memory: a borrow checker with no lifetime syntax, through `from` clauses and single-region `view` types.

Scored:
- Design claude's second bet, an effect system as the major feature: right, down to most of the effects listed.
- Memory safety without a garbage collector, simpler than Rust: right.
- The name Keel, or Forge: wrong.
- The backend: not known until stage 2.
- Tankun's bet, compile-checked units: not the headline; whether Tally has units at all wasn't in the report.

It also landed near Cyborg without seeing it: borrows naming where they're `from`, like Cyborg's `Lend from $r`, and
stack sizes worked out by the compiler so tasks need no `async`, like Cyborg's D350.
