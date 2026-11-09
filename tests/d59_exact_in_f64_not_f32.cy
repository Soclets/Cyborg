// `0.1` rounds in both, but this one is exact only with more bits.
// trace: D59
@config(default);

fn.main [] -> void:
    let $a: f64 = 16777217.0;
    let $b: f32 = 16777217.0;
    return void;
end fn.main
