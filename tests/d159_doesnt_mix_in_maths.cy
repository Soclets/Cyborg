// trace: D159

fn.main [none] -> void:
    let $reading: f64.ieee = 1.5;
    let $x: f64.stable = 1.5;
    let $y: f64.ieee = $reading + $x;
    return void;
end fn.main
