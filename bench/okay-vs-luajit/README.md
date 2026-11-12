# `OKAY\n` against LuaJIT

Two real programs from the Computer Language Benchmarks Game, built by `OKAY\n` and run on LuaJIT, in standard 10.9's
kinds. Timed with hyperfine on 2026-11-12 by design claude, after GAIM claimed LuaJIT would "very likely win".

- **n-body:** the sun and four planets, simulated for 1,000,000 steps. Floats, structs in an array, and a square root
  in the inner loop. Every version prints `-0.169075164` and `-0.169086185`, the Benchmarks Game's published answers.
- **fannkuch-redux, n = 11:** every permutation of 11 items, flipping each until 0 comes first. Integer arrays, swaps
  and branches. Every version prints `556355` and `Pfannkuchen(11) = 51`.

Cyborg's floats are `f64.stable`, the reproducible kind, and every version's output was checked before timing.

## Whole programs, startup included

In milliseconds: the mean, ± the standard deviation, and the min. Each had 2 warm-up runs and at least 10 measured ones.

| Language | Kind | Mode | n-body | fannkuch-redux |
|---|---|---|---|---|
| Cyborg | Normal | `Working --opt=O0` | 1831.9 ± 16.3 (min 1819.2) | 6821.3 ± 91.6 (min 6637.2) |
| Cyborg | Normal | `Working --opt=O1` | 233.4 ± 1.3 (min 231.4) | 1305.7 ± 12.7 (min 1291.9) |
| Cyborg | Normal | `Working --opt=O3` | 236.5 ± 0.8 (min 235.0) | 1226.9 ± 7.3 (min 1215.6) |
| Cyborg | Normal | `FinalSafe` | 236.6 ± 1.1 (min 234.5) | 1239.2 ± 26.9 (min 1216.0) |
| Cyborg | Normal | `FinalPerf` | 236.7 ± 0.9 (min 234.7) | 1250.2 ± 13.1 (min 1230.9) |
| Cyborg | Normal | `FinalShit` | 236.9 ± 0.9 (min 235.2) | 1231.4 ± 5.1 (min 1226.1) |
| Cyborg | Normal | `Literal` | 1651.2 ± 14.7 (min 1618.4) | 6836.2 ± 101.1 (min 6720.2) |
| Cyborg | Ceil.source | `FinalPerf` | 116.8 ± 0.5 (min 115.8) | 1253.7 ± 17.1 (min 1233.2) |
| LuaJIT | Normal | JIT on | 54.8 ± 0.5 (min 53.4) | 2481.0 ± 13.3 (min 2460.8) |
| LuaJIT | Purpose | JIT on | 212.5 ± 13.0 (min 206.3) | 2518.0 ± 26.4 (min 2491.0) |
| LuaJIT | Ceil.source | JIT on | 36.0 ± 4.3 (min 34.6) | 1674.1 ± 23.1 (min 1653.8) |
| LuaJIT | Normal | JIT off (`-joff`) | 617.6 ± 4.6 (min 612.2) | 16254.0 ± 192.4 (min 15940.1) |

What's behind them:

- **fannkuch-redux: Cyborg is twice as fast as LuaJIT's normal code,** 1239 against 2481 ms in `FinalSafe`, with every
  bounds check on every array access still in, and 1.35× faster than LuaJIT's hand-tuned FFI version. Cyborg's own
  Ceil version gains nothing over its Normal one, since LLVM already finds those tricks.
- **n-body: LuaJIT wins, because Cyborg has no square root.** `std::math::sqrt` isn't designed yet, so Cyborg computes
  it with Newton's method, about eight divisions in a loop, where LuaJIT's `math.sqrt` is one hardware instruction.
  LuaJIT's Purpose version runs Cyborg's exact algorithm, Newton's method included, and takes 212 ms against Cyborg's
  237, so doing identical work, they're close. Cyborg's Ceil version starts each pair's Newton's method from its root
  in the step before, which halves its time, but nothing replaces the instruction.
- **The safety checks cost nothing measurable here:** `FinalSafe`, `FinalPerf` and `FinalShit` are within noise of each
  other. That's partly the known bug where `FinalPerf` and `FinalShit` still keep overflow checks
  (`bench/okay-vs-rust`), and partly that these loops' checks are cheap.
- **`Literal` and `Working --opt=O0` are slow on purpose:** no optimizing at all.
- **Cyborg has no Purpose rows.** Purpose reads its size from the command line, and `os`'s arguments aren't designed.
  Neither program can be worked out while compiling, so the Normal rows do the real work anyway.
- **`FinalSmall` doesn't build**, the same compiler bug as in `bench/okay-vs-node`.

## Versions and machine

- An Apple M5, macOS 27.2, with a load average of about 2 on 10 cores, so not a quiet machine.
- `OKAY\n` at the compiler clone's `12483ed9`, a debug build of the compiler, from before the compiler stopped on
  2026-11-11. So the Cyborg programs are written in that day's syntax: the manifest's `okay` line instead of
  `dialect original` (D310), `size:` instead of `room:` (D386), `std::array` instead of `std::coll::array` (D378), and
  bare parameters and loops (D393). Spelling doesn't change speed.
- LuaJIT 2.1.1787165859, hyperfine 2.0.0.
- The raw results are in `results/`: `nbody.csv`, `fannkuch.csv`, and `ceil.csv` for Cyborg's two Ceil versions.

## Running it again

Build each Cyborg program from `cyborg/` with that compiler, as in `okay comp --FinalSafe nbody.cy`, or
`okay comp --Working --opt=O1 nbody.cy`, and the Ceil ones with `--FinalPerf`. Run LuaJIT's from `lua/`, with
`luajit lua/nbody_purpose.lua 1000000` and `luajit lua/fannkuch_purpose.lua 11` for Purpose. Time them all with
`hyperfine -N --warmup 2 --min-runs 10`.
