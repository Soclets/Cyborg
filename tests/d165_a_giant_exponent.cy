// trace: D165, D158
@config(default);

fn.main [] -> void:
    let $a: f64 = exponent ~1.0e99999999999999999999999999;
    return void;
end fn.main
