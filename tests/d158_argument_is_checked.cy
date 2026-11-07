// trace: D158, D20
fn use_ratio [$r: f64[min: 0.0, max: 1.0]] -> void:
    return void;
end fn use_ratio

fn.main [] -> void:
    let $x: f64 = 1.5;
    use_ratio($x);
    return void;
end fn.main
