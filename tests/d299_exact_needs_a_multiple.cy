// An array's length is known, so `.exact` on one that isn't a multiple is refused when compiling.
// trace: D299
import std;

fn.main [none] -> void:
    let mut $xs: std::coll::array<i32>[len: 10 items] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut $sum: simd<i32>[lanes: 4] = all 0;
    for $chunk in $xs.%simd_chunks.exact(lanes: 4):
        change $sum = $sum + $chunk;
    end for
    return void;
end fn.main
