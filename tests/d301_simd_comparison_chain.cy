// A chain of comparisons on `simd` values is listed as undecided (12.2).
// trace: D301
import std;
import os;

fn.main [none] -> void:
    let $a: simd<i32>[lanes: 4] = [1, 8, 3, 6];
    let $b: simd<i32>[lanes: 4] = [5, 5, 5, 5];
    let $m: simd<bool>[lanes: 4] = $a below $b below $a;
    return void;
end fn.main
