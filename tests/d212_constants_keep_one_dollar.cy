// trace: D212, D83
import std;
import os;

const $max: i32 = 100;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $max);
    return void;
end fn.main
