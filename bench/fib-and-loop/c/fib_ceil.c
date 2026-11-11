// Ceil: as fast as C can go. fib(35) by fast doubling, in a handful of steps.
#include <stdio.h>

static void fib_pair(long n, long *f, long *g) { // f = fib(n), g = fib(n + 1)
    if (n == 0) { *f = 0; *g = 1; return; }
    long a, b;
    fib_pair(n / 2, &a, &b);
    long c = a * (2 * b - a), d = a * a + b * b;
    if (n % 2) { *f = d; *g = c + d; } else { *f = c; *g = d; }
}

int main(void) {
    long f, g;
    fib_pair(35, &f, &g);
    printf("%ld\n", f);
    return 0;
}
