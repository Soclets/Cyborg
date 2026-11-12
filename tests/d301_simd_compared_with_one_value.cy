// Comparing a `simd` with one plain value is listed as undecided (12.2).
// trace: D301
import std;
import os;

fn.main [none] -> void:
    let $a: simd<i32>[lanes: 4] = [1, 8, 3, 6];
    let $i: i32 = 2;
    let $m: simd<bool>[lanes: 4] = $a below $i;
    return void;
end fn.main
