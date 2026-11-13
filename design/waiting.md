# Waiting

What's waiting on something else, mostly compiler claude finishing its resume prompt, so it isn't lost. Started on
2026-11-11.

## Design

- **Limbs, left:** the C interface itself, a limb's API and how it's versioned. What limbs are and can do is D334 to
  D338.


- **Handles for shared data:** a `std` slots collection with run-time checked handles, for things like game objects,
  after the compiler's up (12.2).

## compiler claude

Starts on 2026-11-13, which is about getting the compiler back up, with no big design that day (owner). It uses
`design/compiler-resume-prompt.md`: its own remaining work first, against the tests at
`37a5f03b`, then a separate catch-up prompt for D288 onwards.

Waited for Lyftar's embedded clang (Lyftar's D55, Cyborg's D308), which is ready since 2026-11-12: `lyftar::compile_c`
(SystemTarget, the C file, the object, CCompileOptions with include folders and defines). Lyftar renamed `LinkTarget` to
`SystemTarget` (its D56), so the compiler's `link` calls need the new name. CORE-MATH also needs `fma` and `round`.

- `FinalPerf` and `FinalShit` still keep overflow checks, where 10.1 says none (`bench/okay-vs-rust`).
- `FinalSmall` doesn't build: LLVM 23 has removed the `Oz` pipeline name (`bench/okay-vs-node`).
- Compile speed from D168: incremental, parallel and cached builds, which shape the compiler, so they come before
  programs get big (rule 10).
- Decisions since it stopped, D288 to D409, picked up under rule 16. Then its own plan: generics, inline assembly, the
  rest of baking and `use`, then SIMD.

## Lyftar design

Once compiler claude has finished its resume prompt (owner, 2026-11-13), wake Lyftar design and pass on a gap: D233's
check that inline assembly writes no register missing from `writes:` or `clobbers:` can't be done yet, since Lyftar can't
say which registers an instruction defines. Compiler claude asks for something like
`lyftar::registers_written(triple, cpu, assembly_text) -> Result<Vec<String>>`, running MC's assembly parser and giving
each MCInst's explicit and implicit defs, flags included, as register names. Until then, wrong clobbers compile.

## oracle claude

Idle since 2026-11-12, with the generator's coverage finished. When woken: build D403 to D409 (`lanes: max` per target,
`shift.left` and `rotate.left`, `std::math::sqrt`, crashes flushing only free writers, and the 10 error messages
that say "traps" saying "crashes"), then channels when Tankun picks the run-loop rewrite.

## bench

- Rerun `bench/okay-vs-luajit`'s n-body once the compiler has `std::math::sqrt` or `inline.asm`, since today's numbers
  measure Newton's method against a hardware instruction.
- Bench `OKAY\n` against Go properly, in standard 10.9's kinds, once compiler claude's catch-up is done (owner,
  2026-11-13): CPU work, and networking once `@concur` and `os::events` are built, since GAIM says Go wins on both.
