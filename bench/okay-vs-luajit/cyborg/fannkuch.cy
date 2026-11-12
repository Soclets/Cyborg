import std;
import os;
use std::io::fmt_print as fmt_print;

// fannkuch-redux, from the Computer Language Benchmarks Game, for n = 11: every
// permutation of 11 items, flipping the front of each until 1 comes first, with a
// checksum and the most flips. Positions count from 1 in Cyborg, so the 0-based
// algorithm reads item i at pos: i + 1.

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    let $n: i64 = 11;
    let mut $perm1: std::array<i64>[len: 11 items] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut $perm: std::array<i64>[len: 11 items] = all 0;
    let mut $count: std::array<i64>[len: 11 items] = all 0;
    let mut $max_flips: i64 = 0;
    let mut $checksum: i64 = 0;
    let mut $sign: i64 = 1;
    let mut $r: i64 = $n;
    let mut $running: bool = true;
    while $running:
        while $r != 1:
            change $count[pos: $r] = $r;
            change $r = $r - 1;
        end while
        let mut $i: i64 = 0;
        while $i below $n:
            change $perm[pos: $i + 1] = $perm1[pos: $i + 1];
            change $i = $i + 1;
        end while
        let mut $flips: i64 = 0;
        let mut $k: i64 = $perm[pos: 1];
        while $k != 0:
            let mut $lo: i64 = 0;
            let mut $hi: i64 = $k;
            while $lo below $hi:
                let $t: i64 = $perm[pos: $lo + 1];
                change $perm[pos: $lo + 1] = $perm[pos: $hi + 1];
                change $perm[pos: $hi + 1] = $t;
                change $lo = $lo + 1;
                change $hi = $hi - 1;
            end while
            change $flips = $flips + 1;
            change $k = $perm[pos: 1];
        end while
        if $flips above $max_flips:
            change $max_flips = $flips;
        end if
        change $checksum = $checksum + $sign x $flips;
        change $sign = 0 - $sign;
        let mut $next: bool = false;
        while $running and not $next:
            if $r == $n:
                change $running = false;
            else:
                let $first: i64 = $perm1[pos: 1];
                let mut $j: i64 = 0;
                while $j below $r:
                    change $perm1[pos: $j + 1] = $perm1[pos: $j + 2];
                    change $j = $j + 1;
                end while
                change $perm1[pos: $r + 1] = $first;
                change $count[pos: $r + 1] = $count[pos: $r + 1] - 1;
                if $count[pos: $r + 1] above 0:
                    change $next = true;
                else:
                    change $r = $r + 1;
                end if
            end if
        end while
    end while
    fmt_print(Lend.mut $out, "{}{newln}Pfannkuchen({}) = {}{newln}", $checksum, $n, $max_flips);
    os::done(Give $out);
    return void;
end fn.main
