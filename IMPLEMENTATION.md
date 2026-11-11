# Cyborg — Implementation rules

The rules for the code that implements Cyborg: the oracle and the compiler. `DECISIONS.md` records each decision with the
owner's words; this file collects the rules. A hook prints it into every session at startup and after compaction. When a
rule changes, `DECISIONS.md` gets a new entry and this file is updated to match.

## Sessions

- **design claude**: the decisions, `DECISIONS.md`, `IMPLEMENTATION.md` and the standard.
- **oracle claude**: the oracle, in `oracle/`, and the tests, in `tests/`.
- **compiler claude**: the compiler, in `compiler/`, once Lyftar is ready.
- **check claude**: attacks the compiler, in `check/`, once the compiler exists. Its generator can start earlier.

Each works in its own clone, on `main` with no branch, and runs `git pull --rebase` before pushing.

## Rules

1. **Language.** Rust only, and LLVM only through Lyftar. No C++ here: what LLVM needs goes in Lyftar. (D60, D61)
   Tools are Rust too, in a `tools/` crate. The exceptions are one-line commands in config and scratch work that is never
   committed. (D69)
   Third-party C that `std` needs is allowed, in its own folder, with its original license, and no C of our own. (D161)
2. **Where.** Only design claude edits `DECISIONS.md`, `IMPLEMENTATION.md` and the standard.
3. **The oracle stays apart.** compiler claude reads the oracle's tests, never `oracle/src`. If the compiler copied the
   oracle, their bugs would agree.
   check claude reads neither's source: it runs both as black boxes, and reports findings without sending fixes. A difference
   is a bug in one of them, and the standard decides which, or `DECISIONS.md` until it exists. If that's silent, it's a question for the owner. Its generator
   builds programs from a syntax tree with one printer, and tracks what it doesn't generate, since all passing doesn't mean
   covered. (D68)
4. **No design in code.** Code follows `standard/`. Nothing undecided is guessed, and anything in `standard/12-not-decided.md` gives a
   `NotDesignedYet` error, with the question going to the owner through design claude.
5. **Tests.** Output matches the oracle byte for byte, except `f64.fast`, which only has to be close, and may crash where the
   oracle doesn't. (D163) Nothing is pushed while a test fails.
6. **A bad program never crashes the compiler.** No `unwrap`, `expect` or panic on a path real programs take. If the compiler
   itself fails, it says so and how to report it.
7. **Style.** `rustfmt`, `clippy`, and comments that say why. Every crate has these lints (D65):
   `#![forbid(unsafe_code)]`, `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` and
   `#![warn(clippy::pedantic)]`.
8. **Commits.** `kind.folder.narrowing: sentence (owner: "their words"; D1)`. Kinds: `decision`, `design`, `feature`, `fix`,
   `test`, `house`. Each commit ends with its session's trailer, such as
   `Co-Authored-By: oracle claude <noreply@anthropic.com>`.
9. **Cores.** On the owner's 10-core Mac: design claude 1, oracle claude 3, compiler claude 3. That covers builds and
   tests, as in `cargo build -j 3`, `CARGO_BUILD_JOBS=3` and `cargo test -- --test-threads=3`. (D279)
10. **Optimizations from the start.** The compiler is built with its optimizations as it goes, never as a plain version sped up
    later. The oracle stays plain. The owner learnt this reimplementing Lua in SPARK.
    The first compiler has LLVM's standard optimizations and Precog dropping checks, and compiles fast: incremental,
    parallel, cached, and with a lighter `Working` build. (D168)
11. **Prompting.** Before prompting a subagent or a new session, a session reads the current prompting guide for that AI, in
    full and itself, and again after every compaction. Messaging another session isn't prompting, and isn't written like
    prompting: write it like one colleague talking to another.
12. **Stop at 8:30 PM.** Every session stops then, and suggests the owner stops too.
13. **Verus.** The compiler's checks for ownership, bounds, failure coverage, shield containment, and the `@` and `%` rules are
    proven with Verus. The oracle stays plain. (D66, D67)
14. **Scratchpad.** Scratch space is for what can be lost. Anything needed later goes in the repository and gets committed. (D70)
15. **Downloads are free.** A dependency or tool is chosen on maturity, safety, maintenance and speed. A less mature or less safe
    option never wins because the better one needs installing. If an install needs the owner's approval, ask for it. (D70)
16. **Keep up with decisions.** Every pull, read any new entries in `DECISIONS.md`. A small one in your area, like a renamed
    word or a new refusal, just build it. A big one, like a change to most of your tests or how a whole part works, build
    after finishing what you're doing, and ask design claude if it's unclear. Design claude messages only for what's urgent,
    needs explaining, or to wake an idle session. (D284)

## Tracing

Code that carries out a decision says so with a comment, in the oracle and the compiler alike:

```rust
// trace: D13
```

A check fails on a tag that names no decision, and lists the decisions nothing traces yet.

## Naming

The code uses Cyborg's own words, so the code, the decisions and the error messages all say the same thing.

- **Full words.** `expression`, not `expr`.
- **One name per concept.**
- **Cyborg's words over textbook words.** `narrowing`, `shield`, `limb`.
- **Rust's casing**, which `clippy` enforces.
- **Tests are named after their decision.**
