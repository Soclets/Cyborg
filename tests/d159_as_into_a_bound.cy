// trace: D159

fn.main [none] returns void:
    let $reading: f64.ieee set_to 1.5;
    let $x: f64.stable[min: 0.0, max: 2.0] set_to $reading as f64.stable[min: 0.0, max: 2.0];
    return void;
end fn.main
