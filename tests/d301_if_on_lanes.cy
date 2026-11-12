// `if` needs one `bool`, so a `simd<bool>` says whether it means every lane or any lane.
// trace: D301
fn.main [none] -> void:
    let $a: simd<i32>[lanes: 2] = [1, 2];
    let $b: simd<i32>[lanes: 2] = [1, 3];
    if $a == $b:
        return void;
    end if
    return void;
end fn.main
