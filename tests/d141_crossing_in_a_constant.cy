// trace: D141, D83
import std;
import os;

type.new user_id set_to i64;

const $root: user_id set_to 1;
const $plain: i64 set_to $root as i64;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $plain);
    return void;
end fn.main
