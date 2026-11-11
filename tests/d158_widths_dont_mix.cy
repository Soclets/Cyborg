// trace: D158

fn.main [none] returns void:
    let $x: f32.stable set_to 0.5;
    let $y: f64.stable[min: 0.0, max: 1.0] set_to $x;
    return void;
end fn.main
