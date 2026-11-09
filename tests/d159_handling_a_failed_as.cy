// Nothing says what a failed `as` is called, so nothing can name it yet.
// trace: D159
@config(default);

fn.main [] -> void:
    let $x: f64.ieee = 1.5;
    let $clean: f64 = ($x as f64)?;
    return void;
end fn.main
