// trace: D161, D29
fn half [$x: f64] -> f64:
    requires $x at_least 2.0;
    return $x x 0.5;
end fn half

fn.main [] -> void:
    let $h: f64 = half(1.0);
    return void;
end fn.main
