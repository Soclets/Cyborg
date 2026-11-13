# `OKAY\n` against C

n-body and fannkuch-redux from the Computer Language Benchmarks Game, plus two Purpose benchmarks, each isolating
one thing those programs lean on, built by `OKAY\n` in every build mode and by clang at every optimization level, in
every kind standard 10.9 has, and measured the six ways D426 asks. Run on 2026-11-13 by design claude.

## The benchmarks

- **n-body:** the sun and four planets, 1,000,000 steps. Floats, square roots and small arrays. Every build prints
  `-0.169075164` and `-0.169086185`.
- **fannkuch-redux, n = 11:** every permutation of 11 items, flipping each. Integer arrays, swaps and branches. Every
  build prints `556355` and `Pfannkuchen(11) = 51`.
- **Purpose, square root and division:** n-body's inner cost alone, 20,000,000 rounds of `dt / (d2 * sqrt(d2))`, with
  `d2` growing each round. Every build prints `15635.647143706`.
- **Purpose, reversing in place:** fannkuch's inner cost alone, 30,000,000 reversals of the front of an 11-item array.
  Every build prints `149999996`.

Each kind, as 10.9 has them:

- **Normal:** plain code, no hand-tuning.
- **Purpose:** the two isolating programs above.
- **Ceil.source:** hand-tuned, single-threaded. In n-body, each field in its own local array, so every body stays in
  registers; in fannkuch, a permutation starting with 0 is skipped, and each flip puts the old first item straight into
  place.
- **Ceil.machine:** Ceil.source tuned for this Mac, `--machine.cpu=native` and `-mcpu=native`.
- **Ceil.program:** Ceil.source built with a profile of its own run. Only C has these rows: Cyborg's `--profile` waits
  on Lyftar, so it gives NotBuiltYet.

Cyborg ran in all eight of its modes, except that `Literal` refuses `--machine.cpu=`. C ran at `-O0` to `-Oz`, plus
"checked": `-O3 -fsanitize=signed-integer-overflow,bounds -fsanitize-trap=all`, which traps on integer overflow and
out-of-bounds indexes much as `FinalSafe` does, though unlike `FinalSafe` it checks nothing about floats. C's Ceil rows
are at `-O3` only, its fastest level.

## What stands out

- **n-body: Cyborg is 8% faster than plain C.** `FinalPerf` takes 21.1 ms against C's 22.9 at `-O3`. At the ceiling
  they tie, between 20.6 and 20.9 ms whichever kind.
- **fannkuch: they tie at the ceiling.** Cyborg's Ceil.machine `FinalPerf` takes 1,116 ms against C's 1,145, and C's
  profile-guided build 1,108, which Cyborg can't build yet. On Normal, plain C at `-O3` (1,399 ms) beats Cyborg's
  `FinalPerf` (1,459) by 4%.
- **Checked loops ran faster than unchecked ones, in both languages.** fannkuch's Normal `FinalSafe` takes 1,267 ms
  against `FinalPerf`'s 1,459, and C checked takes 1,295 against 1,399. Reversing in place takes 36.9 ms checked
  against 55.5 unchecked in Cyborg, and 36.6 against 52.2 in C. Both go through LLVM, so it looks like LLVM reshapes the
  unchecked swap loop in a way that's slower on this chip. Nobody has looked into why yet.
- **`FinalSafe` is expensive on floats:** 71.2 ms against 21.1 on n-body, and 29.0 against 15.7 on square roots,
  since `f64.stable` checks every result for NaN and infinity, and `sqrt` checks its input isn't negative. C's checked
  builds check nothing about floats, so they stay at full speed, and the rows aren't comparable there.
- **Purpose, square root and division:** Cyborg's 15.7 ms against C's 14.3 to 15.5, within 10%.
- **Compile time:** `OKAY\n` takes 70 to 106 ms from cold for an optimized build, and 12 to 18 ms with the cache warm.
  clang, which has no cache, takes 34 to 148 ms every time.
- **Size:** Cyborg's programs are about 70 KB against C's 33 to 50 KB, and its `Working` and `Literal` builds 150 to 200
  KB. `FinalSmall` is no smaller than `FinalPerf` here, so most of the size is the runtime, not the program.
- **Peak memory** is 1.5 to 1.7 MB for everything, which is the process itself, with Cyborg slightly lower.
- **Debug builds:** Cyborg's `Working --opt=O0` is 6.6 times slower than C's `-O0` on n-body, and 2.8 on fannkuch.

## All results

Run time is the whole program, startup included: the mean, ± the standard deviation, and the minimum, from at least 10
runs after 2 warm-ups. Peak memory is the most any run held. Compile time is the mean of 5 builds: for Cyborg, cold with
`OKAY_CACHE` emptied before each build, and warm after one build; clang has no cache, so it has one number. Every build's
output matched the known answers before it was timed.

### n-body

| Language | Kind | Mode | Run time (ms) | Peak memory (KiB) | Size (bytes) | Compile, cold / warm (ms) |
|---|---|---|---|---|---|---|
| Cyborg | Normal | `Working --opt=O0` | 574.3 ± 22.9 (min 562.1) | 1,664 | 202,912 | 34 / 15 |
| Cyborg | Normal | `Working --opt=O1` | 71.9 ± 0.6 (min 70.0) | 1,584 | 152,912 | 76 / 16 |
| Cyborg | Normal | `Working --opt=O3` | 71.1 ± 0.5 (min 70.0) | 1,584 | 152,832 | 98 / 16 |
| Cyborg | Normal | `FinalSafe` | 71.2 ± 0.6 (min 70.4) | 1,584 | 136,336 | 97 / 16 |
| Cyborg | Normal | `FinalPerf` | 21.1 ± 0.1 (min 20.9) | 1,536 | 70,240 | 101 / 14 |
| Cyborg | Normal | `FinalShit` | 21.2 ± 0.1 (min 20.9) | 1,536 | 70,240 | 95 / 13 |
| Cyborg | Normal | `FinalSmall` | 37.0 ± 12.5 (min 34.0) | 1,536 | 70,576 | 100 / 14 |
| Cyborg | Normal | `Literal` | 307.5 ± 0.4 (min 306.8) | 1,648 | 169,888 | 33 / 17 |
| Cyborg | Ceil.source | `Working --opt=O0` | 653.8 ± 4.3 (min 646.7) | 1,664 | 202,928 | 38 / 17 |
| Cyborg | Ceil.source | `Working --opt=O1` | 90.9 ± 1.6 (min 87.2) | 1,600 | 169,424 | 78 / 18 |
| Cyborg | Ceil.source | `Working --opt=O3` | 87.0 ± 1.2 (min 84.7) | 1,584 | 169,360 | 101 / 16 |
| Cyborg | Ceil.source | `FinalSafe` | 86.7 ± 1.2 (min 82.7) | 1,584 | 152,832 | 101 / 17 |
| Cyborg | Ceil.source | `FinalPerf` | 20.8 ± 0.1 (min 20.7) | 1,536 | 70,224 | 99 / 15 |
| Cyborg | Ceil.source | `FinalShit` | 20.9 ± 0.3 (min 20.7) | 1,568 | 70,224 | 99 / 14 |
| Cyborg | Ceil.source | `FinalSmall` | 33.7 ± 0.9 (min 32.8) | 1,552 | 70,576 | 86 / 15 |
| Cyborg | Ceil.source | `Literal` | 366.3 ± 12.1 (min 358.5) | 1,664 | 186,400 | 33 / 16 |
| Cyborg | Ceil.machine | `Working --opt=O0 --machine.cpu=native` | 661.7 ± 1.9 (min 659.8) | 1,648 | 202,928 | 37 / 17 |
| Cyborg | Ceil.machine | `Working --opt=O1 --machine.cpu=native` | 90.2 ± 1.2 (min 87.6) | 1,584 | 169,424 | 80 / 18 |
| Cyborg | Ceil.machine | `Working --opt=O3 --machine.cpu=native` | 86.0 ± 1.1 (min 83.7) | 1,584 | 169,344 | 102 / 17 |
| Cyborg | Ceil.machine | `FinalSafe --machine.cpu=native` | 86.5 ± 0.8 (min 83.9) | 1,584 | 152,832 | 98 / 16 |
| Cyborg | Ceil.machine | `FinalPerf --machine.cpu=native` | 20.8 ± 0.0 (min 20.7) | 1,536 | 70,224 | 100 / 16 |
| Cyborg | Ceil.machine | `FinalShit --machine.cpu=native` | 21.6 ± 1.1 (min 20.8) | 1,600 | 70,224 | 100 / 15 |
| Cyborg | Ceil.machine | `FinalSmall --machine.cpu=native` | 33.1 ± 0.3 (min 32.8) | 1,600 | 70,576 | 86 / 15 |
| C | Normal | `-O0` | 86.5 ± 0.6 (min 85.6) | 1,712 | 50,016 | 39 |
| C | Normal | `-O1` | 30.8 ± 0.3 (min 30.3) | 1,696 | 49,968 | 43 |
| C | Normal | `-O2` | 27.9 ± 0.2 (min 27.6) | 1,712 | 49,968 | 47 |
| C | Normal | `-O3` | 22.9 ± 0.2 (min 22.6) | 1,696 | 50,000 | 128 |
| C | Normal | `-O3` checked | 23.2 ± 2.3 (min 22.5) | 1,696 | 50,000 | 129 |
| C | Normal | `-Os` | 28.2 ± 0.2 (min 28.0) | 1,696 | 50,000 | 46 |
| C | Normal | `-Oz` | 30.9 ± 0.1 (min 30.6) | 1,712 | 50,032 | 44 |
| C | Ceil.source | `-O3` | 20.9 ± 1.4 (min 20.4) | 1,696 | 33,736 | 146 |
| C | Ceil.source | `-O3` checked | 20.6 ± 0.2 (min 20.4) | 1,696 | 33,736 | 146 |
| C | Ceil.machine | `-O3 -mcpu=native` | 20.6 ± 0.2 (min 20.3) | 1,696 | 33,736 | 144 |
| C | Ceil.program | `-O3`, profile | 20.8 ± 0.4 (min 20.3) | 1,680 | 33,736 | 147 |
| C | Ceil.machine.program | `-O3 -mcpu=native`, profile | 20.8 ± 0.3 (min 20.4) | 1,680 | 33,736 | 148 |

### fannkuch-redux

| Language | Kind | Mode | Run time (ms) | Peak memory (KiB) | Size (bytes) | Compile, cold / warm (ms) |
|---|---|---|---|---|---|---|
| Cyborg | Normal | `Working --opt=O0` | 6,757.8 ± 83.0 (min 6,637.7) | 1,552 | 87,248 | 22 / 18 |
| Cyborg | Normal | `Working --opt=O1` | 1,308.8 ± 19.2 (min 1,283.6) | 1,536 | 70,240 | 72 / 14 |
| Cyborg | Normal | `Working --opt=O3` | 1,203.6 ± 8.6 (min 1,194.0) | 1,536 | 70,208 | 96 / 14 |
| Cyborg | Normal | `FinalSafe` | 1,267.4 ± 10.4 (min 1,254.9) | 1,536 | 70,208 | 106 / 14 |
| Cyborg | Normal | `FinalPerf` | 1,458.6 ± 18.7 (min 1,443.5) | 1,536 | 70,208 | 90 / 12 |
| Cyborg | Normal | `FinalShit` | 1,448.0 ± 4.0 (min 1,442.4) | 1,536 | 70,208 | 103 / 13 |
| Cyborg | Normal | `FinalSmall` | 1,513.8 ± 29.4 (min 1,489.5) | 1,536 | 70,272 | 87 / 13 |
| Cyborg | Normal | `Literal` | 5,420.9 ± 91.5 (min 5,250.2) | 1,536 | 87,248 | 20 / 14 |
| Cyborg | Ceil.source | `Working --opt=O0` | 6,109.0 ± 30.6 (min 6,075.9) | 1,552 | 87,248 | 22 / 15 |
| Cyborg | Ceil.source | `Working --opt=O1` | 1,275.2 ± 18.3 (min 1,253.1) | 1,552 | 86,752 | 72 / 14 |
| Cyborg | Ceil.source | `Working --opt=O3` | 1,272.1 ± 22.2 (min 1,240.6) | 1,536 | 86,720 | 94 / 14 |
| Cyborg | Ceil.source | `FinalSafe` | 1,290.5 ± 16.2 (min 1,272.5) | 1,536 | 70,208 | 95 / 15 |
| Cyborg | Ceil.source | `FinalPerf` | 1,127.8 ± 1.8 (min 1,125.0) | 1,536 | 70,208 | 95 / 15 |
| Cyborg | Ceil.source | `FinalShit` | 1,125.7 ± 3.0 (min 1,122.9) | 1,536 | 70,208 | 97 / 14 |
| Cyborg | Ceil.source | `FinalSmall` | 1,163.6 ± 2.5 (min 1,160.4) | 1,536 | 70,272 | 82 / 14 |
| Cyborg | Ceil.source | `Literal` | 5,342.7 ± 81.6 (min 5,240.3) | 1,536 | 87,248 | 21 / 15 |
| Cyborg | Ceil.machine | `Working --opt=O0 --machine.cpu=native` | 6,102.5 ± 69.9 (min 6,011.3) | 1,552 | 87,248 | 21 / 14 |
| Cyborg | Ceil.machine | `Working --opt=O1 --machine.cpu=native` | 1,259.7 ± 19.2 (min 1,239.0) | 1,536 | 86,736 | 73 / 14 |
| Cyborg | Ceil.machine | `Working --opt=O3 --machine.cpu=native` | 1,248.0 ± 5.9 (min 1,235.3) | 1,536 | 86,720 | 93 / 14 |
| Cyborg | Ceil.machine | `FinalSafe --machine.cpu=native` | 1,268.6 ± 8.4 (min 1,260.0) | 1,536 | 70,208 | 96 / 15 |
| Cyborg | Ceil.machine | `FinalPerf --machine.cpu=native` | 1,116.1 ± 2.4 (min 1,112.6) | 1,536 | 70,208 | 93 / 16 |
| Cyborg | Ceil.machine | `FinalShit --machine.cpu=native` | 1,114.6 ± 3.4 (min 1,111.5) | 1,536 | 70,208 | 95 / 14 |
| Cyborg | Ceil.machine | `FinalSmall --machine.cpu=native` | 1,165.9 ± 1.3 (min 1,164.1) | 1,536 | 70,272 | 85 / 14 |
| C | Normal | `-O0` | 2,423.5 ± 14.5 (min 2,402.3) | 1,696 | 33,568 | 34 |
| C | Normal | `-O1` | 1,400.7 ± 4.2 (min 1,395.1) | 1,696 | 33,616 | 38 |
| C | Normal | `-O2` | 1,392.7 ± 33.3 (min 1,363.6) | 1,680 | 33,568 | 43 |
| C | Normal | `-O3` | 1,398.8 ± 20.6 (min 1,380.2) | 1,680 | 33,568 | 44 |
| C | Normal | `-O3` checked | 1,294.9 ± 17.0 (min 1,275.3) | 1,696 | 33,568 | 45 |
| C | Normal | `-Os` | 1,442.0 ± 23.4 (min 1,412.7) | 1,696 | 33,616 | 40 |
| C | Normal | `-Oz` | 1,389.0 ± 6.3 (min 1,381.0) | 1,696 | 33,568 | 40 |
| C | Ceil.source | `-O3` | 1,133.0 ± 11.0 (min 1,123.1) | 1,696 | 33,568 | 43 |
| C | Ceil.source | `-O3` checked | 1,155.9 ± 22.2 (min 1,136.1) | 1,696 | 33,568 | 46 |
| C | Ceil.machine | `-O3 -mcpu=native` | 1,144.5 ± 26.3 (min 1,117.0) | 1,696 | 33,568 | 44 |
| C | Ceil.program | `-O3`, profile | 1,107.7 ± 9.4 (min 1,097.1) | 1,696 | 33,568 | 45 |
| C | Ceil.machine.program | `-O3 -mcpu=native`, profile | 1,101.9 ± 18.0 (min 1,085.9) | 1,696 | 33,568 | 44 |

### Purpose: square root and division

| Language | Kind | Mode | Run time (ms) | Peak memory (KiB) | Size (bytes) | Compile, cold / warm (ms) |
|---|---|---|---|---|---|---|
| Cyborg | Purpose | `Working --opt=O0` | 133.9 ± 0.9 (min 132.2) | 1,536 | 70,752 | 18 / 13 |
| Cyborg | Purpose | `Working --opt=O1` | 29.1 ± 0.6 (min 27.8) | 1,568 | 70,256 | 69 / 13 |
| Cyborg | Purpose | `Working --opt=O3` | 29.0 ± 0.7 (min 27.6) | 1,568 | 70,224 | 93 / 13 |
| Cyborg | Purpose | `FinalSafe` | 29.0 ± 0.7 (min 27.8) | 1,568 | 70,224 | 92 / 13 |
| Cyborg | Purpose | `FinalPerf` | 15.7 ± 0.2 (min 15.5) | 1,664 | 70,224 | 91 / 13 |
| Cyborg | Purpose | `FinalShit` | 15.9 ± 1.3 (min 15.4) | 1,600 | 70,224 | 91 / 13 |
| Cyborg | Purpose | `FinalSmall` | 15.7 ± 0.3 (min 15.1) | 1,648 | 70,240 | 82 / 13 |
| Cyborg | Purpose | `Literal` | 124.1 ± 2.0 (min 121.6) | 1,536 | 70,752 | 20 / 14 |
| C | Purpose | `-O0` | 61.3 ± 2.8 (min 45.7) | 1,680 | 33,440 | 37 |
| C | Purpose | `-O1` | 15.0 ± 0.6 (min 14.4) | 1,696 | 33,440 | 36 |
| C | Purpose | `-O2` | 15.0 ± 0.3 (min 14.6) | 1,696 | 33,440 | 39 |
| C | Purpose | `-O3` | 15.5 ± 1.5 (min 14.5) | 1,696 | 33,440 | 37 |
| C | Purpose | `-O3` checked | 14.5 ± 0.2 (min 14.2) | 1,696 | 33,440 | 39 |
| C | Purpose | `-Os` | 14.6 ± 0.3 (min 14.2) | 1,680 | 33,440 | 38 |
| C | Purpose | `-Oz` | 14.3 ± 0.1 (min 14.2) | 1,696 | 33,440 | 42 |

### Purpose: reversing in place

| Language | Kind | Mode | Run time (ms) | Peak memory (KiB) | Size (bytes) | Compile, cold / warm (ms) |
|---|---|---|---|---|---|---|
| Cyborg | Purpose | `Working --opt=O0` | 369.3 ± 12.7 (min 353.6) | 1,536 | 70,736 | 19 / 13 |
| Cyborg | Purpose | `Working --opt=O1` | 36.0 ± 0.5 (min 35.1) | 1,536 | 70,240 | 70 / 13 |
| Cyborg | Purpose | `Working --opt=O3` | 36.9 ± 0.3 (min 36.0) | 1,536 | 70,208 | 92 / 13 |
| Cyborg | Purpose | `FinalSafe` | 36.9 ± 0.4 (min 36.0) | 1,536 | 70,208 | 96 / 16 |
| Cyborg | Purpose | `FinalPerf` | 55.5 ± 0.5 (min 54.7) | 1,568 | 70,208 | 93 / 14 |
| Cyborg | Purpose | `FinalShit` | 53.2 ± 0.3 (min 52.8) | 1,536 | 70,208 | 96 / 14 |
| Cyborg | Purpose | `FinalSmall` | 55.0 ± 0.3 (min 54.5) | 1,600 | 70,352 | 83 / 14 |
| Cyborg | Purpose | `Literal` | 291.3 ± 3.5 (min 283.3) | 1,536 | 70,736 | 19 / 14 |
| C | Purpose | `-O0` | 140.9 ± 0.5 (min 139.9) | 1,680 | 33,568 | 35 |
| C | Purpose | `-O1` | 52.4 ± 0.3 (min 51.9) | 1,696 | 33,568 | 38 |
| C | Purpose | `-O2` | 52.9 ± 3.3 (min 51.8) | 1,680 | 33,568 | 39 |
| C | Purpose | `-O3` | 52.2 ± 0.2 (min 51.6) | 1,680 | 33,568 | 39 |
| C | Purpose | `-O3` checked | 36.6 ± 1.2 (min 34.6) | 1,680 | 33,568 | 40 |
| C | Purpose | `-Os` | 52.1 ± 0.2 (min 51.7) | 1,680 | 33,568 | 38 |
| C | Purpose | `-Oz` | 53.1 ± 0.3 (min 52.6) | 1,696 | 33,568 | 37 |

## Setup

- An Apple M5, macOS 27.2.1, with a load average of 1.7 to 2.3 while running, with compiler claude and oracle claude
  stopped.
- `OKAY\n` at the compiler clone's `4130bf78` plus that day's work in progress, a release build from 12:36.
- Apple clang 21.0.0 (clang-2100.4.1.1), `llvm-profdata` from Xcode, and hyperfine 2.0.0.
- The Cyborg programs are in `cyborg/`, the C ones in `c/`, C's profiles in `c/profiles/`, and the raw numbers in
  `results/results.csv`, with every build's command and output in `results/builds.json`.

## Running it again

A scratch script did the work, kept out of the repository under rule 1. For each build, it timed the compile with
hyperfine, 5 runs, cold and warm for Cyborg, moved the program aside, and checked its output. Then it timed every
correct build of each benchmark together with `hyperfine -N --warmup 2 --min-runs 10`. The commands are in
`results/builds.json`. A first run earlier the same day was stopped and thrown away: its C Purpose builds only read
their size from the command line, which isolates nothing, and its C Ceil.source n-body was slower than plain C.
