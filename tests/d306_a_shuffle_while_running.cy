// A shuffle's list is fixed when compiling, so positions from a variable use `.%lookup`.
// trace: D304, D306
fn.main [none] -> void:
    let $v: simd<i32>[lanes: 4] = [1, 2, 3, 4];
    let $n: i32 = 2;
    let $r: simd<i32>[lanes: 1] = $v.%shuffle(pos: [$n]);
    return void;
end fn.main
