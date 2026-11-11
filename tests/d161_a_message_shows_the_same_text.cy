// trace: D161, D29

fn half [$x: f64.stable] returns f64.stable:
    requires $x at_least 2.0;
    return $x x 0.5;
end fn half

fn.main [none] returns void:
    let $h: f64.stable = half(1.0);
    return void;
end fn.main
