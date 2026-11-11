// trace: D149, D123
import std;
import os;

type.new user_id = i32;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = 1;
    let $u: user_id = $n as user_id;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
