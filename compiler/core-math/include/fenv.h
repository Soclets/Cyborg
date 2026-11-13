/* std's freestanding <fenv.h> for compiling CORE-MATH (D271, D308): rounding is always to the nearest, and the
   exception flags do nothing, since Cyborg reads none of them. */
#ifndef CYBORG_FENV_H
#define CYBORG_FENV_H
#define FE_TONEAREST 0
#define FE_DOWNWARD 1
#define FE_UPWARD 2
#define FE_TOWARDZERO 3
#define FE_INVALID 1
#define FE_DIVBYZERO 2
#define FE_OVERFLOW 4
#define FE_UNDERFLOW 8
#define FE_INEXACT 16
static inline int fegetround(void) { return FE_TONEAREST; }
static inline int feraiseexcept(int excepts) { (void)excepts; return 0; }
static inline int fetestexcept(int excepts) { (void)excepts; return 0; }
static inline int feclearexcept(int excepts) { (void)excepts; return 0; }
#endif
