// trace: D158, D20
struct gauge:
    ratio: f64[min: 0.0, max: 1.0];
end struct gauge

fn.main [] -> void:
    let $x: f64 = 1.5;
    let $g: gauge = gauge[ratio = $x];
    return void;
end fn.main
