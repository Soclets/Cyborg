// trace: D158

fn.main [none] returns void:
    let $a: f64.fast set_to 1.5;
    let $b: f64.stable set_to 1.5;
    let $c: f64.stable set_to $a + $b;
    return void;
end fn.main
