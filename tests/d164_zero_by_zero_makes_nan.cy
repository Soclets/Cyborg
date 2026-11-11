// trace: D164, D158

fn.main [none] returns void:
    let $zero: f64.stable set_to 0.0;
    let $a: f64.stable set_to $zero div $zero;
    return void;
end fn.main
