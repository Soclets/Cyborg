// trace: D159
@config(default);

fn.main [] -> void:
    let $x: f64.stable = 1.5;
    let $reading: f64.ieee = $x;
    return void;
end fn.main
