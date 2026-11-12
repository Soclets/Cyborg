// An operator between a `simd` and one plain value is listed as undecided (12.2).
// trace: D291
import std;
import os;

fn.main [none] -> void:
    let $a: simd<i32>[lanes: 4] = [1, 8, 3, 6];
    let $c: simd<i32>[lanes: 4] = $a + 1;
    return void;
end fn.main
