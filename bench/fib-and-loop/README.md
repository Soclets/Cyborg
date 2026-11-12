# fib and a loop: the oracle against C, LuaJIT, Node and Java

Two small programs in five languages, in standard 10.9's three kinds, timed with hyperfine on 2026-11-11.

- **fib:** recursive `fib(35)`, about 30 million calls. Its Purpose is the cost of a function call.
- **loop:** adding 1 to 100,000,000 in a loop. Its Purpose is a loop with an add.

Every version prints the same answers, 9227465 and 5000000050000000, and that was checked before anything was timed.

**The oracle is a plain interpreter on purpose** (rule 10): it walks the checked program and checks everything every time. So these numbers say nothing about how fast compiled Cyborg will be. The compiler can't run loops yet, so it isn't here.

## Kinds and modes (D77, D78)

Every number says its kind and its mode. The oracle has no build mode, since it isn't a compiled program. For the others, the mode column gives the equivalent: the compiler flags or how the JIT runs.

| Kind | What it's for | How each language's version is written |
|---|---|---|
| Purpose | Isolates one thing | The count comes from the command line, so nothing is known at compile time. The C versions also put an empty `asm volatile` barrier on each result, so clang can't turn a call into a loop or the loop into `n x (n + 1) / 2`. The oracle never folds anything and can't read its arguments yet (`os`'s arguments aren't designed), so its Normal programs are its Purpose ones too. |
| Normal | Code as a normal person writes it | Plain, with the numbers written in, and no tuning or tricks. |
| Ceil | As fast as the language can go | Anything allowed: fib by fast doubling, and the loop as `n x (n + 1) / 2`. Every Ceil time is the program starting, working out a handful of numbers and printing. There's no oracle row, since it's plain on purpose and a Ceil row would say nothing. |

## Whole programs, startup included

Each is the whole process, timed from outside, so the JVM and Node pay their startup like everything else. In milliseconds: the mean, ± the standard deviation, and the min below it.

| Language | Kind | Mode | fib | loop |
|---|---|---|---|---|
| Oracle | Normal and Purpose | no build mode, `cargo build --release` | 3714.9 ± 64.2 (min 3623.6) | 7064.1 ± 73.2 (min 6964.4) |
| C | Purpose | clang `-O2` | 21.1 ± 1.1 (min 18.3) | 24.6 ± 0.6 (min 23.4) |
| C | Purpose | clang `-O3` | 21.0 ± 1.1 (min 18.3) | 24.9 ± 1.8 (min 23.5) |
| C | Normal | clang `-O2` | 14.3 ± 0.4 (min 13.6) | 0.9 ± 0.1 (min 0.8) |
| C | Normal | clang `-O3` | 14.0 ± 0.2 (min 13.7) | 0.9 ± 0.1 (min 0.8) |
| C | Ceil.machine | clang `-O3 -march=native` | 1.0 ± 0.3 (min 0.8) | 0.9 ± 0.0 (min 0.8) |
| LuaJIT | Purpose | JIT on, by default | 34.1 ± 1.3 (min 32.1) | 54.1 ± 0.4 (min 53.4) |
| LuaJIT | Normal | JIT on, by default | 34.1 ± 1.5 (min 32.0) | 54.5 ± 0.6 (min 53.3) |
| LuaJIT | Ceil.source | JIT on, by default | 1.5 ± 0.6 (min 1.1) | 1.3 ± 0.1 (min 1.1) |
| Node (V8) | Purpose | no flags | 65.2 ± 5.5 (min 60.3) | 73.8 ± 0.7 (min 72.7) |
| Node (V8) | Normal | no flags | 67.9 ± 8.3 (min 61.3) | 73.9 ± 0.8 (min 72.6) |
| Node (V8) | Ceil.source | no flags | 20.2 ± 2.1 (min 17.9) | 20.0 ± 0.8 (min 18.9) |
| Java | Purpose | default flags | 32.3 ± 0.4 (min 31.5) | 49.4 ± 10.5 (min 46.1) |
| Java | Normal | default flags | 32.6 ± 1.0 (min 31.4) | 48.0 ± 1.5 (min 45.6) |
| Java | Ceil.source | `-XX:TieredStopAtLevel=1 -XX:+UseSerialGC -Xshare:auto` | 20.8 ± 2.0 (min 19.5) | 21.4 ± 2.8 (min 18.4) |

What's behind some of these:

- **C's Normal fib beats its Purpose fib.** At `-O2`, clang keeps one of fib's two calls and turns the other into a loop, so the Normal build makes about half the calls. The Purpose build makes every one, at roughly 0.7 ns a call.
- **C's Normal loop is startup alone.** clang works the whole sum out at compile time, and the program only prints 5000000050000000, as its disassembly shows. That's what plain C gets, so it stays in Normal. The Purpose loop is the real loop, at about 0.24 ns a pass.
- **The Ceil rows are each language's startup:** about 1 ms for C and LuaJIT, and about 20 ms for Node and Java.
- **Against the oracle:** on the Purpose rows, C is about 175× faster on fib and 290× on the loop. LuaJIT is about 110× and 130×, Node about 57× and 96×, and Java about 115× and 143×.

## Steady state, inside one process

This is a different question from whole programs, so it's a separate table: how fast each JIT runs the work once it has compiled it, with startup and warm-up left out. Each harness (`java/Steady.java`, `js/steady.js`, `lua/steady.lua`) runs each benchmark 10 times to warm up for Java, 5 for Node and LuaJIT, then times 20 more inside the same process. Times are in milliseconds.

| Language | Kind | Mode | fib | loop |
|---|---|---|---|---|
| Java | Purpose | default flags, warmed up | 12.5 ± 0.2 (min 12.1) | 24.2 ± 0.6 (min 23.6) |
| LuaJIT | Purpose | JIT on, warmed up | 30.0 ± 0.3 (min 29.8) | 52.9 ± 0.9 (min 51.7) |
| Node (V8) | Purpose | no flags, warmed up | 40.6 ± 0.9 (min 39.8) | 53.9 ± 1.0 (min 52.6) |

The raw output is in `results/steady.txt`. Once warmed up, Java's fib is faster than C's Purpose fib. That's most likely HotSpot inlining part of the recursion, which the C version's barriers forbid, so it isn't the same work as C's Purpose row.

## The machine and versions

- An Apple M5 with 10 cores and 24 GB, on macOS 27.0.1 (26A434).
- **Not a quiet machine:** this ran alongside the compiler session's builds and the usual desktop. The load average was about 2.5 on 10 cores throughout (`results/load-before.txt` and `results/load-after.txt`). Each program is single-threaded, though the JVM and Node run short-lived helper threads of their own.
- The oracle at `78d8eff`, built with `cargo build --release`.
- Apple clang 21.0.0 (clang-2100.4.1.1), LuaJIT 2.1.1787165859, Node v24.16.0, OpenJDK 26.0.2.1 (Homebrew), and hyperfine 2.0.0.
- Every program had 3 warm-up runs and at least 10 measured ones, and most had far more, since hyperfine runs fast ones for about 3 seconds. The oracle had 2 warm-up runs and 10 measured ones, since each takes seconds. Each run count and its full summary, peak memory included, is in `results/*.csv`.

## Running it again

From this folder, with hyperfine, LuaJIT, Node, a JDK and clang installed, and the oracle built with `cargo build --release` in `oracle/`:

```
mkdir -p build/java
clang -O2 -o build/fib_normal_O2 c/fib_normal.c
clang -O3 -o build/fib_normal_O3 c/fib_normal.c
clang -O2 -o build/fib_purpose_O2 c/fib_purpose.c
clang -O3 -o build/fib_purpose_O3 c/fib_purpose.c
clang -O3 -march=native -o build/fib_ceil c/fib_ceil.c
clang -O2 -o build/loop_normal_O2 c/loop_normal.c
clang -O3 -o build/loop_normal_O3 c/loop_normal.c
clang -O2 -o build/loop_purpose_O2 c/loop_purpose.c
clang -O3 -o build/loop_purpose_O3 c/loop_purpose.c
clang -O3 -march=native -o build/loop_ceil c/loop_ceil.c
javac -d build/java java/*.java
```

Then the fib programs, all in one hyperfine run:

```
hyperfine -N --warmup 3 --min-runs 10 --export-csv results/fib.csv \
  -n "C Normal -O2" "build/fib_normal_O2" -n "C Normal -O3" "build/fib_normal_O3" \
  -n "C Purpose -O2" "build/fib_purpose_O2 35" -n "C Purpose -O3" "build/fib_purpose_O3 35" \
  -n "C Ceil -O3 -march=native" "build/fib_ceil" \
  -n "LuaJIT Normal" "luajit lua/fib_normal.lua" -n "LuaJIT Purpose" "luajit lua/fib_purpose.lua 35" \
  -n "LuaJIT Ceil" "luajit lua/fib_ceil.lua" \
  -n "Node Normal" "node js/fib_normal.js" -n "Node Purpose" "node js/fib_purpose.js 35" \
  -n "Node Ceil" "node js/fib_ceil.js" \
  -n "Java Normal" "java -cp build/java FibNormal" -n "Java Purpose" "java -cp build/java FibPurpose 35" \
  -n "Java Ceil" "java -XX:TieredStopAtLevel=1 -XX:+UseSerialGC -Xshare:auto -cp build/java FibCeil"
```

The loop is the same with `loop` for `fib`, `Loop` for `Fib`, and `100000000` for `35`, exported to `results/loop.csv`. Then the oracle and the steady state:

```
hyperfine -N --warmup 2 --runs 10 --export-csv results/oracle.csv \
  -n "Oracle fib" "../../oracle/target/release/oracle cyborg/fib.cy" \
  -n "Oracle loop" "../../oracle/target/release/oracle cyborg/loop.cy"
java -cp build/java Steady
node js/steady.js
luajit lua/steady.lua
```

This replaces `bench/oracle-vs-v8`, which was two runs each with `time` against Node, for fun. Its Cyborg programs moved to `cyborg/` here.

The Ceil rows were relabelled `Ceil.source` or `Ceil.machine` on 2026-11-12, when D364 split Ceil. The numbers are unchanged.
