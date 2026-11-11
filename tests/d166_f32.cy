// trace: D166, D159
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $zero: f32.ieee = 0.0;
    let $inf: f32.ieee = 1.0 div $zero;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $inf);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($zero div $zero));
    return void;
end fn.main
