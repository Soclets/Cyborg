// trace: D43, D116
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32 set_to 2 + 3 x 4;
    let $b: i32 set_to (2 + 3) x 4;
    let $c: i32 set_to 10 - 3 - 2;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    return void;
end fn.main
