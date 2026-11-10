// A fix that goes somewhere else, like an import, can't be applied, though the next error's fix still is.
// trace: D207, D82
@config(default);
fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    let $a: f64.stable = 0.1;
    let $b: f64.stable = $a;
    let $c: i32 = 1;
    let $d: i64 = $c;
    return void;
end fn.main
