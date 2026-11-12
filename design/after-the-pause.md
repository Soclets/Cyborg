# After the pause

What's waiting for when the owner unpauses Cyborg, written down on 2026-11-11 so it isn't lost.

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
- Decisions since it stopped, D288 to D365, picked up under rule 16. Then its own plan: generics, inline assembly, the
  rest of baking and `use`, then SIMD.

## oracle claude

Idle since 2026-11-12, with the generator's coverage finished. When woken: build D403 to D407 (`lanes: max` per target,
`shift.left` and `rotate.left`, and `std::math::sqrt`), then channels when Tankun picks the run-loop rewrite.

## bench

- Rerun `bench/okay-vs-luajit`'s n-body once the compiler has `std::math::sqrt` or `inline.asm`, since today's numbers
  measure Newton's method against a hardware instruction.
