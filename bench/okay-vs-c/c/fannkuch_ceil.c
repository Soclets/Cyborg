// fannkuch-redux, Ceil.source: hand-tuned and single-threaded, with the same tricks as the Cyborg
// Ceil version: a permutation starting with 0 is skipped without copying, and each flip puts the
// old first item straight into place instead of re-reading it.
#include <stdio.h>

int main(void) {
    const int n = 11;
    int perm1[16], perm[16], count[16];
    for (int i = 0; i < n; i++) perm1[i] = i;
    int max_flips = 0, checksum = 0, sign = 1, r = n;
    for (;;) {
        while (r != 1) { count[r - 1] = r; r--; }
        int k = perm1[0];
        if (k != 0) {
            for (int i = 1; i < n; i++) perm[i] = perm1[i];
            int flips = 1;
            for (;;) {
                for (int lo = 1, hi = k - 1; lo < hi; lo++, hi--) { int t = perm[lo]; perm[lo] = perm[hi]; perm[hi] = t; }
                int nk = perm[k];
                perm[k] = k;
                k = nk;
                if (k == 0) break;
                flips++;
            }
            if (flips > max_flips) max_flips = flips;
            checksum += sign * flips;
        }
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
