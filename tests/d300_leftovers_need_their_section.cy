// `.leftovers` hands the leftovers to a `leftovers` section, so the loop needs one.
// trace: D300
import std;

fn.main [none] -> void:
    let mut $xs: std::coll::array<i32>[len: 10 items] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut $sum: simd<i32>[lanes: 4] = all 0;
    for $chunk in $xs.std::%simd_chunks.leftovers(lanes: 4):
        change $sum = $sum + $chunk;
    end for
    return void;
end fn.main
