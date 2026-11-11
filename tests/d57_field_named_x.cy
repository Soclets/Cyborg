// `x` is an operator, but a field has no clash, because `.` says what it is.
// trace: D57, D75
import std;
import os;

struct point:
    x: i32;
    y: i32;
end struct point

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $p: point set_to point[x set_to 3, y set_to 4];
    let $n: i32 set_to $p.x x $p.y;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
