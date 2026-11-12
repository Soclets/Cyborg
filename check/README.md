# check

Cyborg's random program generator and the live comparison of the oracle and `OKAY\n` (D313,
rule 3). The generator is written from `standard/`, never from `oracle/src` or `compiler/src`,
so what counts as a valid program comes from the docs. Both are run as black boxes.

```
cargo run -- generate <seed> <count> <folder>
cargo run -- compare <oracle> <okay> <program.cy or folder>...
cargo run -- fuzz <oracle> <okay> <seed> <count> <folder>
cargo run -- coverage
```

- `generate` writes programs from consecutive seeds, with a shared manifest. The same seed always
  gives the same program.
- `compare` runs each program with the oracle, then with `okay run` in every build mode: `Working`
  at each `--opt=` level, `FinalSafe`, `FinalPerf`, `FinalShit`, `FinalSmall` and `Literal`. Every
  mode has to match the oracle byte for byte, except that when the oracle stops on a failed check,
  only the modes that keep every check have to match (10.3, 10.4). A program the oracle calls
  `NotDesignedYet` or `NotBuiltYet` is out of scope, and `f64.fast` is left to judge by hand,
  since it only has to be close (rule 5).
- `fuzz` generates, compares, and shrinks each difference to the smallest program that still shows
  it, written beside it as `<name>.smallest.cy`.
- `coverage` lists what the generator doesn't generate yet, since all passing doesn't mean covered.

A difference is a bug in one of them, and the standard decides which. Findings about the compiler
go to compiler claude without fixes.
