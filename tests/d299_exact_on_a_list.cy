// A list's length is only known while running, so `.exact` checks it as the loop starts.
// trace: D299
import std;

fn.main [none] -> void:
    let $xs: std::coll::list<i32>[len.max: 8 items] = [1, 2, 3];
    let mut $sum: simd<i32>[lanes: 2] = all 0;
    for $chunk in $xs.std::%simd_chunks.exact(lanes: 2):
        change $sum = $sum + $chunk;
    end for
    return void;
end fn.main
