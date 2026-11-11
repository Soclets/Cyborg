// trace: D141
import std;
import os;

type.new user_id set_to i64;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $n: i64 set_to 5;
    let $u: user_id set_to $n as user_id;
    let $m: i64 set_to $u as i64;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $m);
    let $bounded: i64[min: 0, max: 9] set_to 3;
    let $v: user_id set_to $bounded as user_id;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($v as i64));
    return void;
end fn.main
