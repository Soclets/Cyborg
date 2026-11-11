// trace: D141, D20
import std;
import os;

type.new percent set_to i32[min: 0, max: 100];

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $p: percent set_to 40;
    let $n: i32 set_to $p as i32;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
