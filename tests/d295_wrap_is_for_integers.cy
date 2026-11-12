// Wrapping is for integers: floats go to infinity instead, so it means nothing for them.
// trace: D295, D42, D373
fn.main [none] -> void:
    let $a: f64.stable = 1.5;
    let $b: f64.stable = $a minus.wrap 1.0;
    return void;
end fn.main
