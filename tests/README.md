# Tests

Each test is a Cyborg program and its exact result, compared byte for byte with what the oracle does:

- `name.cy` is the program.
- `name.out` is its exact standard output.
- `name.err` is its exact error. A test with one must exit with 1, and any other exits with 0.
- `name.exit` holds the exit code, when it isn't that, as for a program that prints to stderr with
  `std::io::fmt_print.stderr` and ends well.

A missing `.out` or `.err` means that stream stays empty. Tests run from this folder, so errors name the file as `name.cy`.

Every program needs a manifest next to it (D183), and a folder is one project, so the programs here share `CyJet.siml`. A test
of the manifest itself is a folder of its own, like `d228_need_with_no_version/`, holding its `CyJet.siml` and a `main.cy` with
its `.out` or `.err`. The tests run every folder inside this one the same way.

Names start with the decision they test, as in `d13_len_always`, and `// trace: D13` says which decisions a test follows.
`program_` tests are real programs using many at once, and `sketch` is `design/sketch.md`. A refusal shows every error building
finds, in the order they appear, with knock-on ones pointing back at their cause (D207).

Run them with `cargo test` in `oracle/`, and check the traces from the top of the repository with:

```
cargo run --manifest-path tools/Cargo.toml
```

`f64.fast` only has to come close, and may crash where the oracle doesn't (D163). So a compiler's output doesn't have to match
the `d163_` tests that run, and any other test using `.fast` is a refusal, which still matches byte for byte. How close counts as
close isn't decided, so no test prints a `.fast` answer yet.
