// trace: D134, D20
import std;
import os;

type.alias percent set_to i32[min: 0, max: 100];

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: percent set_to 50;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    let $big: i32 set_to 101;
    let $b: percent set_to $big;
    return void;
end fn.main
