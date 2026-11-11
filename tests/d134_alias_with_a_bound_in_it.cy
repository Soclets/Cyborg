// trace: D134, D20
import std;
import os;

type.alias percent = i32[min: 0, max: 100];

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: percent = 50;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    let $big: i32 = 101;
    let $b: percent = $big;
    return void;
end fn.main
