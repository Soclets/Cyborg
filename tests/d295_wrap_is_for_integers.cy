// Wrapping is for integers.
// trace: D295, D42
fn.main [none] -> void:
    let $a: f64.stable = 1.5;
    let $b: f64.stable = $a minus.wrap 1.0;
    return void;
end fn.main
