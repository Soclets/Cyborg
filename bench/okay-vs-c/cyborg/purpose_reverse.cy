import std;
import os;
use std::io::fmt_print as fmt_print;

// Purpose: reversing the front of an array in place, fannkuch's inner cost, alone. 30,000,000
// rounds over 11 items, reversing the first 2, then 3, up to 11, and again, with a checksum of
// the first item so the work can't be skipped.

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    let mut $a: std::coll::array<i64>[len: 11 items] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut $checksum: i64 = 0;
    let mut $k: i64 = 2;
    for $round in range[from: 1, to: 30_000_000]:
        let mut $lo: i64 = 0;
        let mut $hi: i64 = $k - 1;
        while $lo below $hi:
            let $t: i64 = $a[offset.items: $lo];
            change $a[offset.items: $lo] = $a[offset.items: $hi];
            change $a[offset.items: $hi] = $t;
            change $lo = $lo + 1;
            change $hi = $hi - 1;
        end while
        change $checksum = $checksum + $a[offset.items: 0];
        if $k == 11:
            change $k = 2;
        else:
            change $k = $k + 1;
        end if
    end for
    fmt_print(Lend.mut $out, "{}{newln}", $checksum);
    os::done(Give $out);
    return void;
end fn.main
