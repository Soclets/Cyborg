// A fix that goes somewhere else, like an import, can't be applied, though the next error's fix still is.
// trace: D207, D82
import os;
fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    let $a: f64.stable = 0.1;
    let $b: f64.stable = $a;
    let $c: i32 = 1;
    let $d: i64 = $c;
    return void;
end fn.main
