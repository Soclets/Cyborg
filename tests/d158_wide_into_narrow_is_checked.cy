// trace: D158, D20

fn.main [none] returns void:
    let $w: f64.stable[min: 0.0, max: 3.0] set_to 2.5;
    let $n: f64.stable[min: 1.0, max: 2.0] set_to $w;
    return void;
end fn.main
