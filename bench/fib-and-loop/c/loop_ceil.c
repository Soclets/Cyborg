// Ceil: as fast as C can go. The sum of 1 to n is n x (n + 1) / 2.
#include <stdio.h>

int main(void) {
    long n = 100000000;
    printf("%ld\n", n * (n + 1) / 2);
    return 0;
}
