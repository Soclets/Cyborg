// trace: D164

fn.main [none] returns void:
    let $a: f64.stable = 7.5;
    let $b: f64.stable = $a mod.trunc 2.0;
    return void;
end fn.main
