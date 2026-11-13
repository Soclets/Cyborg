# Cyborg alone on the Uno R4 WiFi

Three integer workloads on the board's RA4M1, a Cortex-M4 at 48 MHz, timed with its cycle counter, `DWT CYCCNT`, in
each of `OKAY\n`'s fixed build modes, on 2026-11-13. Nothing else is compared: this is how Cyborg itself runs on the
chip. Every answer was checked against Python's before counting it.

- **Collatz:** the total steps for every start from 1 to 30,000, in `i64`. Loops, branches, division by 2, and 64-bit
  maths on a 32-bit chip. Answer 2,864,311.
- **Primes:** primes below 20,000 by trial division, in `i32`. Mostly division. Answer 2,262.
- **xorshift:** 1,000,000 rounds of a 32-bit xorshift generator. Shifts and `xor.bit`. Answer 2,318,261,108.

| Mode | Collatz | Primes | xorshift | Code size (bytes) |
|---|---|---|---|---|
| `FinalSafe` | 52,926,449 cycles (1,103 ms) | 7,155,966 (149 ms) | 3,187,502 (66.4 ms) | 920 |
| `FinalPerf` | 37,806,600 (788 ms) | 6,610,005 (138 ms) | 3,187,502 (66.4 ms) | 1,304 |
| `FinalSmall` | 47,168,913 (983 ms) | 7,299,915 (152 ms) | 6,999,999 (145.8 ms) | 500 |
| `Literal` | 246,529,264 (5,136 ms) | 25,383,629 (529 ms) | 67,999,028 (1,416.6 ms) | 1,160 |

What's behind them:

- **`FinalSafe`'s checks cost 40% on Collatz and 8% on primes,** against `FinalPerf`. Collatz does a checked `+` and
  `x` on every step, in 64 bits, where each check is a few extra instructions; primes spends its time dividing, which
  checks barely add to. xorshift has nothing to check, so the two are identical.
- **`FinalSmall` is the smallest, at 500 bytes, and close to `FinalSafe`'s speed,** except on xorshift, where it keeps
  the loop rolled up, at exactly 7 cycles a round.
- **`Literal` is 5 to 21 times slower,** on purpose: it does exactly what the code says, with no optimizing.
- **Overflow on `x` is still checked in `FinalPerf`** (the `smull` and compare in primes' `$d x $d`), since 10.5 leaves
  `-` and `x` undecided, and rule 4 keeps an undecided stop.

## How it ran

The program leaves each answer and its cycle count in RAM from `0x2000_0000`, and a debugger reads them through the
board's CMSIS-DAP interface with `openocd`, since serial printing isn't built yet. After that, uploads fail until the
board is unplugged and plugged back in.

## Compiler bugs it found

- **`shift.left` that drops bits is treated as overflow.** 3.7 says bits that fall off are dropped, but `FinalSafe`
  trapped on xorshift's first round, and `FinalPerf`, which assumes no overflow, let LLVM replace the whole loop with a
  trap instruction. The program masks the bits off before shifting, which gives the same answer.
- The two `memset` problems in `examples/uno_r4_matrix`.
