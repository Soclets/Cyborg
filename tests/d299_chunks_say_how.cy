// A bare `simd_chunks` is refused, asking which of the three ways the leftovers go.
// trace: D299, D300
import std;

fn.main [none] -> void:
    let mut $xs: std::array<i32>[len: 10 items] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut $sum: simd<i32>[lanes: 4] = all 0;
    for $chunk in $xs.%simd_chunks(lanes: 4):
        change $sum = $sum + $chunk;
    end for
    return void;
end fn.main
