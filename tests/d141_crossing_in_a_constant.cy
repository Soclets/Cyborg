// trace: D141, D83
import std;
import os;

type.new user_id = i64;

const $root: user_id = 1;
const $plain: i64 = $root as i64;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $plain);
    return void;
end fn.main
