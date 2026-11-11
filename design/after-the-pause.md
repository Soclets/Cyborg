# After the pause

What's waiting for when the owner unpauses Cyborg, written down on 2026-11-11 so it isn't lost.

## Design

- **Arenas, next:** whether a list from an arena still ends with `done`, which frees nothing, with ending the arena
  early a failed check (Claude's pick), or borrows the arena and needs no `done`. Then the placeholder names: `os::heap`,
  `std::list.done`, `std::text.done` and `std::text.from`. Memory up to there is D314 to D328.
- **How limbs connect to the compiler:** what a limb is to `OKAY\n`, and what it can see and change. The Precog limb,
  `@csup` and every `inline.@limb::lang` wait on it.
- **Concurrency:** threads, sharing data, and anything like goroutines or async (12.8).
- **A secret type** for constant-time code (12.7).

- **"Merging with the Machine"**: the owner wants it on the README and GitHub later, not yet (2026-11-11).

## compiler claude

- `FinalPerf` and `FinalShit` still keep overflow checks, where 10.1 says none (`bench/okay-vs-rust`).
- `FinalSmall` doesn't build: LLVM 23 has removed the `Oz` pipeline name (`bench/okay-vs-node`).
- Compile speed from D168: incremental, parallel and cached builds, which shape the compiler, so they come before
  programs get big (rule 10).
- Decisions since it stopped, D288 to D328, picked up under rule 16. Then its own plan: generics, inline assembly, the
  rest of baking and `use`, then SIMD.

## oracle claude

- Decisions since it stopped, D288 to D328, under rule 16. Its test `d42_minus_wrap_isnt_decided` is out of date since
  D295.
- Then the random generator and the live comparison with the compiler, in `check/` (D313).
