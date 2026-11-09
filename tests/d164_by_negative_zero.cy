// trace: D164, D158, D160
@config(default);

fn.main [] -> void:
    let $zero: f64 = -0.0;
    let $a: f64 = 1.0 div $zero;
    return void;
end fn.main
