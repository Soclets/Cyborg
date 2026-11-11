// The sum isn't known to stay inside, so going back in is checked.
// trace: D158, D20

fn.main [none] returns void:
    let $a: f64.stable[min: 0.0, max: 1.0] = 0.75;
    let $b: f64.stable[min: 0.0, max: 1.0] = $a + $a;
    return void;
end fn.main
