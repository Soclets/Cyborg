// A lane follows its type's rules exactly, and the failure says which lane.
// trace: D293
fn.main [none] -> void:
    let $a: simd<i32>[lanes: 3] = [1, 2147483647, 3];
    let $b: simd<i32>[lanes: 3] = $a + $a;
    return void;
end fn.main
