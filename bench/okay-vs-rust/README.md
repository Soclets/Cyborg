# `OKAY\n` against Rust

fib(35) and the 100M loop, built by `OKAY\n` in its build modes, against Rust in all three kinds of standard 10.9. Timed
with hyperfine on 2026-11-11 by design claude, after a benchmark against Rust that ChatGPT asked for.

Every version prints the same answers, 9227465 and 5000000050000000, checked before timing.

## Whole programs, startup included

In milliseconds: the mean, ± the standard deviation, and the min. Each had 3 warm-up runs and at least 10 measured ones.

| Language | Kind | Mode | fib | loop |
|---|---|---|---|---|
| Cyborg | Normal | `Working --opt=O0` | 38.3 ± 2.9 (min 34.1) | 188.2 ± 12.7 (min 166.2) |
| Cyborg | Normal | `Working --opt=O1` | 21.1 ± 0.9 (min 19.4) | 28.5 ± 5.2 (min 22.9) |
| Cyborg | Normal | `FinalSafe` | 21.3 ± 1.1 (min 19.1) | 25.4 ± 2.3 (min 22.9) |
| Cyborg | Normal | `FinalPerf` | 21.2 ± 0.8 (min 19.4) | 24.9 ± 1.8 (min 23.4) |
| Cyborg | Normal | `FinalShit` | 21.4 ± 1.0 (min 19.1) | 25.4 ± 1.9 (min 23.8) |
| Cyborg | Normal | `Literal` | 34.2 ± 1.3 (min 30.9) | 269.5 ± 13.2 (min 238.6) |
| Cyborg | Ceil | `FinalPerf` | 0.9 ± 0.2 (min 0.8) | 1.0 ± 0.1 (min 0.8) |
| Rust | Normal | `opt-level=0` | 37.0 ± 0.3 (min 36.5) | 109.5 ± 4.7 (min 106.6) |
| Rust | Normal | `opt-level=3` | 14.3 ± 0.3 (min 13.7) | 1.2 ± 0.1 (min 1.1) |
| Rust | Normal | `opt-level=3`, overflow checks on | 22.3 ± 0.8 (min 20.3) | 24.0 ± 0.4 (min 23.4) |
| Rust | Purpose | `opt-level=3` | 24.8 ± 0.6 (min 23.9) | 32.2 ± 5.9 (min 24.8) |
| Rust | Purpose | `opt-level=3`, overflow checks on | 23.0 ± 0.4 (min 22.6) | 56.2 ± 1.3 (min 54.7) |
| Rust | Ceil | `opt-level=3`, `target-cpu=native`, fat LTO | 1.1 ± 0.1 (min 1.0) | 1.2 ± 0.1 (min 1.1) |

Every Rust build used `codegen-units=1`. The Ceil one also used `panic=abort`.

What's behind them:

- **Cyborg ties Rust with overflow checks on**: 21.2 against 22.3 ms on fib, and 24.9 against 24.0 on the loop.
- **Rust without checks wins on Normal**: 14.3 ms on fib, since LLVM turns one of fib's two calls into a loop, and 1.2
  ms on the loop, since it works out the whole sum while compiling, like C did in `bench/fib-and-loop`.
- **That's partly a compiler bug.** `FinalPerf` should have no runtime checks (10.1), but its loop still has the
  overflow check, an `adds` then a `b.vs`, which stops LLVM from folding the loop or reshaping fib. So Cyborg's
  `FinalPerf` and `FinalShit` rows are really "with overflow checks", and match Rust's checked rows. The no-checks
  comparison waits for the fix.
- **Rust Purpose** uses `std::hint::black_box` on every result, so nothing is folded or merged. It reads the count from
  the command line. Cyborg has no Purpose rows, since `os`'s arguments aren't designed.
- **Debug builds:** Cyborg's `Working --opt=O0` is close to Rust's `opt-level=0` on fib, and slower on the loop.
- **`FinalSmall`** isn't here: it fails to build, as `bench/okay-vs-node` says.

## Versions and machine

- An Apple M5, on macOS 27.0.1, with a load average of about 2 on 10 cores, so not a quiet machine.
- `OKAY\n` at the compiler clone's `12483ed9`, a debug build of the compiler, the same binaries as `bench/okay-vs-node`.
  The manifest used the old `okay` line, since that build came before D309 and D310.
- rustc 1.96.1, called directly with `-C` flags, and hyperfine 2.0.0.
- The Cyborg programs are in `bench/fib-and-loop/cyborg/` and `bench/okay-vs-node/cyborg/`, the Rust ones in `rust/`
  here, and the raw results in `results/`.

## Running it again

Each Rust program builds with `rustc -C opt-level=3 -C codegen-units=1 fib_normal.rs`, adding
`-C overflow-checks=on` for the checked rows, and `-C target-cpu=native -C lto=fat -C panic=abort` for Ceil. Purpose
takes the count as its argument: `fib_purpose 35` and `loop_purpose 100000000`. Then time them with
`hyperfine -N --warmup 3 --min-runs 10`.
