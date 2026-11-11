// trace: D158

fn.main [none] returns void:
    let $x: f32.stable set_to 0.75;
    let $y: f32.stable[min: ~0.1, max: 0.5] set_to $x;
    return void;
end fn.main
