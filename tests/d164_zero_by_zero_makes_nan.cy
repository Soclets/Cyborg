// trace: D164, D158

fn.main [none] returns void:
    let $zero: f64.stable = 0.0;
    let $a: f64.stable = $zero div $zero;
    return void;
end fn.main
