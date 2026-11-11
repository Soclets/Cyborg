// trace: D164, D158, D160

fn.main [none] returns void:
    let $zero: f64.stable = -0.0;
    let $a: f64.stable = 1.0 div $zero;
    return void;
end fn.main
