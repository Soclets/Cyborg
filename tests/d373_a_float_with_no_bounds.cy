// Saturating a float whose type writes no bounds has nothing to stick at but infinity, which a
// float can't be, so it isn't decided.
// trace: D373
fn.main [none] -> void:
    let $a: f64.stable = 1.5;
    let $b: f64.stable = $a plus.saturate 1.0;
    return void;
end fn.main
