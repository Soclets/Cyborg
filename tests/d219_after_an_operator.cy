// A line can break after a binary operator, `=` included.
// trace: D219
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 1;
    let $b: i32 = $a +
        2;
    let $c: i32 =
        $b x 3;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    return void;
end fn.main
