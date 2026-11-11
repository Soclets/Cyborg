// trace: D212, D83
import std;
import os;

const $max: i32 set_to 100;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $max);
    return void;
end fn.main
