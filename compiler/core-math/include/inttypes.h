/* std's freestanding <inttypes.h> for compiling CORE-MATH (D271, D308): only the format macros its debugging code
   names, which is never compiled in. */
#ifndef CYBORG_INTTYPES_H
#define CYBORG_INTTYPES_H
#include <stdint.h>
#define PRId64 "lld"
#define PRIx64 "llx"
#endif
