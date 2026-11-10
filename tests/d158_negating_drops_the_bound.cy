// trace: D158, D116
@config(default);

fn.main [] -> void:
    let $a: f64.stable[min: 0.0, max: 1.0] = 0.5;
    let $b: f64.stable[min: 0.0, max: 1.0] = -$a;
    return void;
end fn.main
