// trace: D216, D83
import std;
import os;

const $max: i32 = 100;
const $twice := $max x 2;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    const $half := $max div.trunc 2;
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $twice, $half);
    return void;
end fn.main
