// Purpose: a loop with an add. The count comes from the command line, and an empty asm barrier
// keeps the sum in a register each time round, so clang can't replace the loop with
// n x (n + 1) / 2 or vectorize it.
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 100000000;
    long sum = 0;
    for (long i = 1; i <= n; i++) {
        sum += i;
        __asm__ volatile("" : "+r"(sum));
    }
    printf("%ld\n", sum);
    return 0;
}
