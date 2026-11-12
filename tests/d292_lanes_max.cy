// `lanes: max` is the target's own width, and the oracle builds for no target.
// trace: D292, D286
fn.main [none] -> void:
    let $d: simd<i32>[lanes: max] = all 0;
    return void;
end fn.main
