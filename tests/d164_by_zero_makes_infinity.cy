// trace: D164, D158
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $zero: f64.stable set_to 0.0;
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    let $a: f64.stable set_to 1.0 div $zero;
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
