// trace: D164, D97
@config(default);

fn.main [] -> void:
    let $a: f64.stable = 7.5;
    let $b: f64.stable = $a mod 2.0;
    return void;
end fn.main
