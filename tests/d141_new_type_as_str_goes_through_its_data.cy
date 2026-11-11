// trace: D141, D122
import std;
import os;

type.new user_id set_to i64;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $u: user_id set_to 42;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($u as i64));
    return void;
end fn.main
