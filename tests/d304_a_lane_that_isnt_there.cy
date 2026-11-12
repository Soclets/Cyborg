// A shuffle's lanes are checked when compiling.
// trace: D304
fn.main [none] -> void:
    let $v: simd<i32>[lanes: 4] = [1, 2, 3, 4];
    let $r: simd<i32>[lanes: 1] = $v.%shuffle(pos: [5]);
    return void;
end fn.main
