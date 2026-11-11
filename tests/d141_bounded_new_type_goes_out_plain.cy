// trace: D141, D20
import std;
import os;

type.new percent = i32[min: 0, max: 100];

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $p: percent = 40;
    let $n: i32 = $p as i32;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
