// trace: D158, D20
fn.main [] -> void:
    let mut $t: f64[min: 0.0, max: 1.0] = 0.5;
    let $x: f64 = 1.5;
    change $t = $x;
    return void;
end fn.main
