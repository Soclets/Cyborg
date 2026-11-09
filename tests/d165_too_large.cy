// trace: D165, D158
@config(default);

fn.main [] -> void:
    let $a: f64 = exponent ~1.0e400;
    return void;
end fn.main
