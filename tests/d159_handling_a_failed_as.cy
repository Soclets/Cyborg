// Nothing says what a failed `as` is called, so nothing can name it yet.
// trace: D159

fn.main [none] -> void:
    let $x: f64.ieee = 1.5;
    let $clean: f64.stable = ($x as f64.stable)?;
    return void;
end fn.main
