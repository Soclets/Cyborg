// trace: D161, D29
@config(default);

fn half [$x: f64.stable] -> f64.stable:
    requires $x at_least 2.0;
    return $x x 0.5;
end fn half

fn.main [] -> void:
    let $h: f64.stable = half(1.0);
    return void;
end fn.main
