# CORE-MATH

Correctly rounded maths for `std::math` (D161, D256), from [CORE-MATH](https://core-math.gitlabpages.inria.fr/) at
commit `018fa97` (2026-11-10), under its own MIT license in `LICENSE`. `OKAY\n` compiles these files itself, for whatever
target it's building, through the clang inside Lyftar (D308), and only when a program uses them.

`src/` holds CORE-MATH's files unchanged, at their own paths. `include/` holds the small freestanding headers `std` ships
for them, since `std` has no C library (D271, D308): rounding is always to the nearest, and exception flags do nothing.
