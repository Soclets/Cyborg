// trace: D164, D97

fn.main [none] returns void:
    let $a: f64.stable set_to 7.5;
    let $b: f64.stable set_to $a mod 2.0;
    return void;
end fn.main
