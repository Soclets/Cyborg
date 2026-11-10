// trace: D158
@config(default);

fn.main [] -> void:
    let $x: f32.stable = 0.75;
    let $y: f32.stable[min: ~0.1, max: 0.5] = $x;
    return void;
end fn.main
