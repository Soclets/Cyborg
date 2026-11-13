// Purpose: reversing the front of an array in place, fannkuch's inner cost, alone. 30,000,000
// rounds over 11 items, reversing the first 2, then 3, up to 11, and again, with a checksum of
// the first item so the work can't be skipped.
#include <stdint.h>
#include <stdio.h>

int main(void) {
    int64_t a[11];
    for (int i = 0; i < 11; i++) a[i] = i;
    int64_t checksum = 0;
    int64_t k = 2;
    for (long round = 0; round < 30000000; round++) {
        for (int64_t lo = 0, hi = k - 1; lo < hi; lo++, hi--) { int64_t t = a[lo]; a[lo] = a[hi]; a[hi] = t; }
        checksum += a[0];
        k = k == 11 ? 2 : k + 1;
    }
    printf("%lld\n", (long long)checksum);
    return 0;
}
