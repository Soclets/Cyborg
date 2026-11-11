// Normal: adding 1 to 100,000,000 in a loop, written plainly.
#include <stdio.h>

int main(void) {
    long sum = 0;
    for (long i = 1; i <= 100000000; i++) sum += i;
    printf("%ld\n", sum);
    return 0;
}
