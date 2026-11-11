// trace: D164, D163, D158

fn.main [none] -> void:
    let $zero: f64.fast = 0.0;
    let $a: f64.fast = 1.0 div $zero;
    return void;
end fn.main
