// `$reading as f64` is what's decided, and nothing's said about going back.
// trace: D159

fn.main [none] -> void:
    let $x: f64.stable = 1.5;
    let $reading: f64.ieee = $x as f64.ieee;
    return void;
end fn.main
