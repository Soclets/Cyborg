# After the pause

What's waiting for when the owner unpauses Cyborg, written down on 2026-11-11 so it isn't lost.

## Design

- **Limbs, left:** the C interface itself, a limb's API and how it's versioned. What limbs are and can do is D334 to
  D338.


## compiler claude

Waited for Lyftar's embedded clang (Lyftar's D55, Cyborg's D308), which is ready since 2026-11-12: `lyftar::compile_c`
(SystemTarget, the C file, the object, CCompileOptions with include folders and defines). Lyftar renamed `LinkTarget` to
`SystemTarget` (its D56), so the compiler's `link` calls need the new name. CORE-MATH also needs `fma` and `round`.

- `FinalPerf` and `FinalShit` still keep overflow checks, where 10.1 says none (`bench/okay-vs-rust`).
- `FinalSmall` doesn't build: LLVM 23 has removed the `Oz` pipeline name (`bench/okay-vs-node`).
- Compile speed from D168: incremental, parallel and cached builds, which shape the compiler, so they come before
  programs get big (rule 10).
- Decisions since it stopped, D288 to D329, picked up under rule 16. Then its own plan: generics, inline assembly, the
  rest of baking and `use`, then SIMD.

## oracle claude

- Decisions since it stopped, D288 to D329, under rule 16. Its test `d42_minus_wrap_isnt_decided` is out of date since
  D295.
- Then the random generator and the live comparison with the compiler, in `check/` (D313).
