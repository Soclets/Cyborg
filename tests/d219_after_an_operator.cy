// A line can break after a binary operator, `=` included.
// trace: D219
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32 set_to 1;
    let $b: i32 set_to $a +
        2;
    let $c: i32 set_to
        $b x 3;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    return void;
end fn.main
