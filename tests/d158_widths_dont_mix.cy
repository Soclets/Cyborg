// trace: D158
@config(default);

fn.main [] -> void:
    let $x: f32.stable = 0.5;
    let $y: f64.stable[min: 0.0, max: 1.0] = $x;
    return void;
end fn.main
