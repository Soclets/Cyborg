// A fix that goes somewhere else, like an import, can't be applied, though the next error's fix still is.
// trace: D207, D82
import os;
fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    let $a: f64.stable set_to 0.1;
    let $b: f64.stable set_to $a;
    let $c: i32 set_to 1;
    let $d: i64 set_to $c;
    return void;
end fn.main
