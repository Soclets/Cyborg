// trace: D164, D83
import std;
import os;

const $half: f64.stable set_to 1.0 div 2.0;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $half);
    return void;
end fn.main
