// `lanes: max` is the target's own width, so a program using it names its target.
// trace: D292, D389
fn.main [none] -> void:
    let $d: simd<i32>[lanes: max] = all 0;
    return void;
end fn.main
