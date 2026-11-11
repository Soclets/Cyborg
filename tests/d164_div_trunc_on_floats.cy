// trace: D164, D54

fn.main [none] returns void:
    let $a: f64.stable set_to 1.0;
    let $b: f64.stable set_to $a div.trunc 2.0;
    return void;
end fn.main
