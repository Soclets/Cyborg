// `as` converts between an array and a `simd` of the same count, so another count is refused.
// trace: D298
import std;

fn.main [none] -> void:
    let $a: std::array<i32>[len: 3 items] = [1, 2, 3];
    let $v: simd<i32>[lanes: 4] = $a as simd<i32>[lanes: 4];
    return void;
end fn.main
