// trace: D164, D54
@config(default);

fn.main [] -> void:
    let $a: f64 = 1.0;
    let $b: f64 = $a div.trunc 2.0;
    return void;
end fn.main
