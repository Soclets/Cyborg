// trace: D158, D116

fn.main [none] returns void:
    let $a: f64.stable[min: 0.0, max: 1.0] set_to 0.5;
    let $b: f64.stable[min: 0.0, max: 1.0] set_to -$a;
    return void;
end fn.main
