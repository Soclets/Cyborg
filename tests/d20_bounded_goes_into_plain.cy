// trace: D20
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32[min: 0, max: 100] = 7;
    let $b: i32 = $a;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    return void;
end fn.main
