// `simd_chunks` only reads, so its chunks are copies and changing one is refused.
// trace: D303
import std;

fn.main [none] -> void:
    let mut $xs: std::array<i32>[len: 10 items] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut $sum: simd<i32>[lanes: 4] = all 0;
    for $chunk in $xs.%simd_chunks.pad(lanes: 4, pad: 0):
        change $chunk = $chunk + $chunk;
    end for
    return void;
end fn.main
