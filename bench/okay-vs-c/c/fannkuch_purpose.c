// fannkuch-redux, Purpose: the Normal program, but n comes from the command line, so nothing
// about it can be worked out while compiling. Run as `fannkuch_purpose 11`.
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    int n = argc > 1 ? atoi(argv[1]) : 11;
    int perm1[16], perm[16], count[16];
    for (int i = 0; i < n; i++) perm1[i] = i;
    int max_flips = 0, checksum = 0, sign = 1, r = n;
    for (;;) {
        while (r != 1) { count[r - 1] = r; r--; }
        for (int i = 0; i < n; i++) perm[i] = perm1[i];
        int flips = 0, k = perm[0];
        while (k != 0) {
            for (int lo = 0, hi = k; lo < hi; lo++, hi--) { int t = perm[lo]; perm[lo] = perm[hi]; perm[hi] = t; }
            flips++;
            k = perm[0];
        }
        if (flips > max_flips) max_flips = flips;
        checksum += sign * flips;
        sign = -sign;
        for (;;) {
            if (r == n) { printf("%d\nPfannkuchen(%d) = %d\n", checksum, n, max_flips); return 0; }
            int first = perm1[0];
            for (int j = 0; j < r; j++) perm1[j] = perm1[j + 1];
            perm1[r] = first;
            if (--count[r] > 0) break;
            r++;
        }
    }
}
