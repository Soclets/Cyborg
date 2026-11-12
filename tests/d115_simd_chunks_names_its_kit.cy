// `simd_chunks` is a list method from `std`, so it says its kit.
// trace: D115, D299
import std;

fn.main [none] -> void:
    let $xs: std::coll::array<i32>[len: 4 items] = [1, 2, 3, 4];
    for $chunk in $xs.%simd_chunks.exact(lanes: 4):
    end for
    return void;
end fn.main
