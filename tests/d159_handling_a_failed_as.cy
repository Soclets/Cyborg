// Nothing says what a failed `as` is called, so nothing can name it yet.
// trace: D159

fn.main [none] returns void:
    let $x: f64.ieee set_to 1.5;
    let $clean: f64.stable set_to ($x as f64.stable)?;
    return void;
end fn.main
