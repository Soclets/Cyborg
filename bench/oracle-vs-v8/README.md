# The oracle against V8

For fun, on 2026-11-10: the oracle against JavaScript on Node, with the same two programs in each.

| Benchmark | Oracle | Node 24.16 (V8) | V8 is |
|---|---|---|---|
| Recursive `fib(35)`, about 30 million calls | 3.57 s | 0.06 s | about 60× faster |
| A loop adding 1 to 100,000,000 | 6.96 s | 0.07 s | about 100× faster |

Both print the same answers: 9227465 and 5000000050000000. Times are wall-clock, the mean of two runs each, on an Apple M5.

**Kind and mode (D77):** Normal kind, since nothing is hand-tuned. There's no build mode, because this is the oracle, not a compiled program. The oracle is a plain interpreter that checks everything every time, on purpose (rule 10), while V8 compiles JavaScript to machine code as it runs. So this says nothing about how fast Cyborg's compiler will be.

**Versions:** timed on the oracle at `4954c96`, built with `cargo build --release`. The `.cy` files have since been updated for D265 and D267, and still print the same answers.

To run it, from this folder:

```
oracle fib.cy
node fib.js
oracle loop.cy
node loop.js
```
