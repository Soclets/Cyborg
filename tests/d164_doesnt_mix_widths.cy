// trace: D164

fn.main [none] returns void:
    let $a: f64.stable = 1.5;
    let $b: f32.stable = 1.5;
    let $c: f64.stable = $a div $b;
    return void;
end fn.main
