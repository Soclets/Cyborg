// trace: D159

fn.main [none] returns void:
    let $reading: f64.ieee set_to 1.5;
    let $x: f64.stable set_to 1.5;
    let $y: f64.ieee set_to $reading + $x;
    return void;
end fn.main
