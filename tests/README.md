# Tests

Each test is a Cyborg program and its exact result, compared byte for byte with what the oracle does:

- `name.cy` is the program.
- `name.out` is its exact standard output.
- `name.err` is its exact error. A test with one must exit with 1, and any other exits with 0.
- `name.exit` holds the exit code, when it isn't that, as for a program that prints to stderr with
  `std::io::fmt_print.stderr` and ends well.

A missing `.out` or `.err` means that stream stays empty. Tests run from this folder, so errors name the file as `name.cy`.

Names start with the decision they test, as in `d13_len_always`, and `// trace: D13` says which decisions a test follows.
`program_` tests are real programs using many at once, and `sketch` is `design/sketch.md`. A refusal test holds exactly one
error, because how many errors a run gives and their order aren't decided (standard 12.10).

Run them with `cargo test` in `oracle/`, and check the traces from the top of the repository with:

```
cargo run --manifest-path tools/Cargo.toml
```

`f64.fast` only has to come close, and may crash where the oracle doesn't (D163). So a compiler's output doesn't have to match
the `d163_` tests that run, and any other test using `.fast` is a refusal, which still matches byte for byte. How close counts as
close isn't decided, so no test prints a `.fast` answer yet.
