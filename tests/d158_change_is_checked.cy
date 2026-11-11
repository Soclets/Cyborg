// trace: D158, D20

fn.main [none] returns void:
    let mut $t: f64.stable[min: 0.0, max: 1.0] set_to 0.5;
    let $x: f64.stable set_to 1.5;
    change $t set_to $x;
    return void;
end fn.main
