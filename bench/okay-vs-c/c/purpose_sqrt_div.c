// Purpose: square root and float division, n-body's inner cost, alone. 20,000,000 rounds of
// dt over (d2 times its square root), with d2 growing a little each round so nothing repeats,
// added up in order.
#include <math.h>
#include <stdio.h>

int main(void) {
    double d2 = 1.0, total = 0.0;
    const double dt = 0.01, grow = 0.000001;
    for (long i = 0; i < 20000000; i++) {
        total += dt / (d2 * sqrt(d2));
        d2 += grow;
    }
    printf("%.9f\n", total);
    return 0;
}
