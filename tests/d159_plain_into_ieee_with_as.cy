// `$reading as f64` is what's decided, and nothing's said about going back.
// trace: D159

fn.main [none] returns void:
    let $x: f64.stable set_to 1.5;
    let $reading: f64.ieee set_to $x as f64.ieee;
    return void;
end fn.main
