// `{}` takes a value in its readable form, so a number needs no `as str[len.max: 20 bytes]`.
// trace: D194, D95
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = 1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
