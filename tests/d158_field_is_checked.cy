// trace: D158, D20
@config(default);

struct gauge:
    ratio: f64.stable[min: 0.0, max: 1.0];
end struct gauge

fn.main [] -> void:
    let $x: f64.stable = 1.5;
    let $g: gauge = gauge[ratio = $x];
    return void;
end fn.main
