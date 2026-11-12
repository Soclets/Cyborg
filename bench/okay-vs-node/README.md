# `OKAY\n` against Node

The first compiled Cyborg numbers: fib(35) and the 100M loop, built by `OKAY\n`, typed `okay`, in every build mode, against JavaScript on
Node (V8). Timed with hyperfine on 2026-11-11 by design claude, after the owner asked for Cyborg against V8 only.

Every version prints the same answers, 9227465 and 5000000050000000, checked before timing.

## Whole programs, startup included

In milliseconds: the mean, ± the standard deviation, and the min. Each had 3 warm-up runs and at least 10 measured ones.

| Language | Kind | Mode | fib | loop |
|---|---|---|---|---|
| Cyborg | Normal | `Working --opt=O0` | 38.6 ± 1.3 (min 35.4) | 192.2 ± 16.7 (min 160.4) |
| Cyborg | Normal | `Working --opt=O1` | 21.3 ± 0.9 (min 18.9) | 23.5 ± 0.5 (min 22.9) |
| Cyborg | Normal | `Working --opt=O3` | 21.7 ± 2.4 (min 19.0) | 23.5 ± 0.4 (min 22.8) |
| Cyborg | Normal | `FinalSafe` | 22.2 ± 1.0 (min 19.5) | 23.4 ± 0.4 (min 22.8) |
| Cyborg | Normal | `FinalPerf` | 22.1 ± 0.9 (min 19.5) | 23.4 ± 0.4 (min 22.8) |
| Cyborg | Normal | `FinalShit` | 22.6 ± 0.9 (min 19.6) | 24.0 ± 0.9 (min 22.9) |
| Cyborg | Normal | `Literal` | 34.1 ± 1.4 (min 30.6) | 250.9 ± 23.1 (min 195.1) |
| Cyborg | Normal | `FinalSmall` | didn't build | didn't build |
| Cyborg | Ceil.source | `FinalPerf` | 1.0 ± 0.1 (min 0.9) | 0.9 ± 0.0 (min 0.8) |
| Node (V8) | Normal | no flags | 65.4 ± 5.5 (min 61.5) | 70.9 ± 0.5 (min 69.9) |
| Node (V8) | Purpose | no flags | 63.9 ± 2.1 (min 60.1) | 70.8 ± 0.6 (min 69.8) |
| Node (V8) | Ceil.source | no flags | 20.1 ± 0.9 (min 17.8) | 19.5 ± 1.1 (min 17.6) |

What's behind them:

- **Cyborg is about 3× faster than V8** on both, in every mode that optimizes, startup included. Most of V8's time here
  is its own startup, about 20 ms, which its Ceil rows show.
- **`FinalSafe` costs nothing here against `FinalPerf`.** Its overflow checks on `+` are still in, and these loops don't
  notice them.
- **Cyborg's Normal loop is a real loop.** Unlike C's at `-O2` in `bench/fib-and-loop`, LLVM didn't fold it into a
  constant: about 0.23 ns a pass, the same as C's Purpose loop.
- **`Literal` does what the code says (D282):** no unrolling, no vectorizing. `Working --opt=O0` is the debugging
  build, and is slow on purpose.
- **There are no Cyborg Purpose rows.** Purpose versions read the count from the command line, and `os`'s arguments
  aren't designed yet. The Normal programs weren't folded, so they measure the same thing here.
- **`FinalSmall` didn't build.** `okay` asks LLVM 23 for its `Oz` pipeline, which LLVM has removed ("use O2 with the
  minsize attribute"). That's a compiler bug, and `--Working --opt=Oz` would hit it too.

## Versions and machine

- An Apple M5, on macOS 27.0.1. The load average was about 2 on 10 cores, so not a quiet machine.
- `okay` at the compiler clone's `12483ed9`, a debug build of the compiler. That only slows compiling: the programs go
  through the same LLVM passes either way.
- That `okay` came before D309 and D310, so the timing used a copy of the manifest with the old `okay 2026y-11m-10d`
  line instead of `dialect original 2026y-11m-10d`. The code is unchanged.
- Node v24.16.0, hyperfine 2.0.0.
- The Normal programs are `bench/fib-and-loop/cyborg/fib.cy` and `loop.cy`, and the Node programs are in
  `bench/fib-and-loop/js/`. The Cyborg Ceil programs are in `cyborg/` here. The raw results are in `results/`.

## Running it again

From `bench/fib-and-loop/cyborg/`, build each mode with `okay comp --FinalSafe fib.cy` and so on, plus
`okay comp --Working --opt=O1 fib.cy` for `Working`, then time them with
`hyperfine -N --warmup 3 --min-runs 10` next to `node ../js/fib_normal.js`. The Ceil programs here build with
`okay comp --FinalPerf`.

The Ceil rows were relabelled `Ceil.source` or `Ceil.machine` on 2026-11-12, when D364 split Ceil. The numbers are unchanged.
