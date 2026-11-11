// trace: D158

fn.main [none] -> void:
    let $a: f64.fast = 1.5;
    let $b: f64.stable = 1.5;
    let $c: f64.stable = $a + $b;
    return void;
end fn.main
