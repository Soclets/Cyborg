// trace: D83, D59
import std;
import os;

const $b: i32 = 60 x 60;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    const $c: i32 = $b + 1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    return void;
end fn.main
