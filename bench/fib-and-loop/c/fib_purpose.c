// Purpose: the cost of a function call. n comes from the command line, so nothing is known at
// compile time, and an empty asm barrier on each result stops clang from turning one of the two
// calls into a loop, so every call in the tree is made.
#include <stdio.h>
#include <stdlib.h>

__attribute__((noinline)) long fib(long n) {
    if (n < 2) return n;
    long a = fib(n - 1);
    __asm__ volatile("" : "+r"(a));
    long b = fib(n - 2);
    __asm__ volatile("" : "+r"(b));
    return a + b;
}

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 35;
    printf("%ld\n", fib(n));
    return 0;
}
