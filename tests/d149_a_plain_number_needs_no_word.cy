// trace: D149, D123
import std;
import os;

type.new user_id set_to i32;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $n: i32 set_to 1;
    let $u: user_id set_to $n as user_id;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
