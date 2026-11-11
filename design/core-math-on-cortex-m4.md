# CORE-MATH on the Uno R4 WiFi's Cortex-M4

Checked by design claude on 2026-11-11, against CORE-MATH at `018fa97` (2026-11-10), with Homebrew's clang 23 targeting
`thumbv7em-none-eabihf -mcpu=cortex-m4`. The question was whether CORE-MATH's `f64` functions need 128-bit integers that a
32-bit CPU can't have.

## 128-bit integers aren't a problem

15 of CORE-MATH's `binary64` functions use a 128-bit integer, among them `sin`, `cos`, `tan`, `log`, `pow`, `atan2` and
`hypot`. Each picks its type the same way:

```c
#if (defined(__clang__) && __clang_major__ >= 14) || (… gcc 14 with _BitInt(128) …)
typedef unsigned _BitInt(128) u128;
#else
typedef unsigned __int128 u128;
#endif
```

`__int128` is refused on the Cortex-M4 ("not supported on this target"), but C23's `_BitInt(128)` compiles there, and a
128-bit multiply needs no helper function. With clang 14 or later, CORE-MATH always takes the `_BitInt` path.

## What the Cortex-M4 does need

`sin`, `cos`, `log` and `exp` compiled to Cortex-M4 objects with stub headers. What they still need:

- **Libc headers.** CORE-MATH includes `fenv.h`, `errno.h`, `math.h`, `inttypes.h` and sometimes `stdio.h`, and `std` is
  freestanding (D271). So `std` ships small headers of its own for compiling CORE-MATH: rounding is always to nearest,
  and exception flags do nothing. Some functions call `printf` in debugging code, which needs it compiled out.
- **Software `f64`.** The Cortex-M4's FPU is single-precision only, so every `f64` add, multiply and compare is a call
  like `__aeabi_dadd`, from compiler-rt's builtins. Any `f64` in Cyborg on this board needs these, not only maths.
- **A correctly rounded `fma` and `round`.** CORE-MATH calls `fma`, and its correct rounding depends on that `fma` being
  exact. On this CPU it's software, so `std` needs a correct one, like musl's or picolibc's, checked against the oracle.
- **Speed.** All of this is software, so `std::math::sin` on `f64` will be slow on the board, but still correctly
  rounded. `f32` versions use the FPU and are much faster.

## Compiling the C at all

`okay` has no C compiler (rule 1 keeps C++ out, and Lyftar has LLVM and lld but not clang). CORE-MATH can be compiled
ahead of time, once per target, with clang 14 or later, and shipped as object files inside `okay`. That needs nothing new
from Lyftar.
