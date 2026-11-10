// A bound is a literal like any other, so rounding says so.
// trace: D158, D59
@config(default);

fn.main [] -> void:
    let $t: f64.stable[min: 0.1, max: 1.0] = 0.5;
    return void;
end fn.main
