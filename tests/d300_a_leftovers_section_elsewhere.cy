// A `leftovers` section belongs only to a `for` over `simd_chunks.leftovers`.
// trace: D300
import std;

fn.main [none] -> void:
    let mut $xs: std::array<i32>[len: 10 items] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut $sum: simd<i32>[lanes: 4] = all 0;
    let mut $n: i32 = 0;
    for $x in $xs:
        change $n = $n + $x;
    leftovers $y:
        change $n = $n + $y;
    end for
    return void;
end fn.main
