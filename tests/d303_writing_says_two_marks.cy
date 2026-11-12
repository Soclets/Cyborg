// `simd_chunks.mut` writes back, so it's called with `.%%`.
// trace: D303, D211, D358
import std;

fn.main [none] -> void:
    let mut $xs: std::coll::array<i32>[len: 10 items] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut $sum: simd<i32>[lanes: 4] = all 0;
    for $chunk in $xs.%simd_chunks.mut.pad(lanes: 4, pad: 0):
        change $chunk = $chunk + $chunk;
    end for
    return void;
end fn.main
