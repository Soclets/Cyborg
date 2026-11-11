// trace: D164, D163, D158

fn.main [none] returns void:
    let $zero: f64.fast set_to 0.0;
    let $a: f64.fast set_to 1.0 div $zero;
    return void;
end fn.main
