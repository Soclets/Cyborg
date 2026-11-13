# Waiting

What's waiting on something else, mostly compiler claude finishing its resume prompt, so it isn't lost. Started on
2026-11-11.

## Design

- **Limbs, left:** the C interface itself, a limb's API and how it's versioned. What limbs are and can do is D334 to
  D338.


- **Handles for shared data:** a `std` slots collection with run-time checked handles, for things like game objects,
  after the compiler's up (12.2).

## compiler claude

Finished `design/compiler-resume-prompt.md` on 2026-11-13, with about 1390 of the snapshot's 1536 tests passing. Next is
`design/compiler-catch-up-prompt.md`: D288 to D411 against today's tests, the older D155, D198, D250, D251, D256 and
D273, and ranges that know loop counters.

## Lyftar design

Once compiler claude has finished its catch-up (owner, 2026-11-13), wake Lyftar design and pass on a gap: D233's
check that inline assembly writes no register missing from `writes:` or `clobbers:` can't be done yet, since Lyftar can't
say which registers an instruction defines. Compiler claude asks for something like
`lyftar::registers_written(triple, cpu, assembly_text) -> Result<Vec<String>>`, running MC's assembly parser and giving
each MCInst's explicit and implicit defs, flags included, as register names. Until then, wrong clobbers compile.

## oracle claude

Idle since 2026-11-12, with the generator's coverage finished. When woken: build D403 to D409 (`lanes: max` per target,
`shift.left` and `rotate.left`, `std::math::sqrt`, crashes flushing only free writers, and the 10 error messages
that say "traps" saying "crashes"), respell the 9 tests that still write `shl`/`shr` (compiler claude lists them:
d49_bits_grouped, the three d49_shift_*, the two d116_bit_*, d340_constant_time_maths, d340_a_shift_by_a_secret and
d379_a_view_cant_grow), and make d38_limb_inside_a_shield's message describe the program rather than the oracle ("from
the oracle, which has no C or assembly in it"), underline the whole `union.@csup::untagged` in
d38_untagged_union_at_the_top rather than only `union.@` (11.1: the run sits under exactly the part in question). Then modules, D412 to D415: every test file gets a `module` line, the
tests grow files that import each other with `pkg::`, and `pub.pkg` is tested. Also D416's `.%viewed_as`, with a test that reads a `struct.packed` and a
`struct.compact` value's raw bytes, which pins the real layouts. Then channels when Tankun picks the run-loop rewrite.

## bench

- Rerun `bench/okay-vs-luajit`'s n-body once the compiler has `std::math::sqrt` or `inline.asm`, since today's numbers
  measure Newton's method against a hardware instruction.
- Bench `OKAY\n` against Go properly, in standard 10.9's kinds, once compiler claude's catch-up is done (owner,
  2026-11-13): CPU work, and networking once `@concur` and `os::events` are built, since GAIM says Go wins on both.
