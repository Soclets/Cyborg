// A bare `offset:` could read as bytes, so a shuffle counting from 0 says `offset.lanes:`.
// trace: D305
fn.main [none] -> void:
    let $v: simd<i32>[lanes: 4] = [1, 2, 3, 4];
    let $r: simd<i32>[lanes: 1] = $v.%shuffle(offset: [0]);
    return void;
end fn.main
