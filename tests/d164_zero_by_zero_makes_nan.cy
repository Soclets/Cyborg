// trace: D164, D158
@config(default);

fn.main [] -> void:
    let $zero: f64 = 0.0;
    let $a: f64 = $zero div $zero;
    return void;
end fn.main
