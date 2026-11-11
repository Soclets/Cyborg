// trace: D141, D122
import std;
import os;

type.new user_id = i64;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $u: user_id = 42;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($u as i64));
    return void;
end fn.main
