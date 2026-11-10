// trace: D164
@config(default);

fn.main [] -> void:
    let $a: f64.stable = 7.5;
    let $b: f64.stable = $a mod.trunc 2.0;
    return void;
end fn.main
