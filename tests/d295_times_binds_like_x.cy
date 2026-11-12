// `plus` and `minus` sit level with `+`, and `times` with `x`, since precedence follows maths.
// trace: D295
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 2;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a + 3 times.wrap 4);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a x 3 plus.wrap 4);
    return void;
end fn.main
