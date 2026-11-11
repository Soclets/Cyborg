// `.%` says what it is, so a method's name doesn't clash.
// trace: D130, D75
import std;
import os;

struct point:
    x: i32;
end struct point

fn point.%x [$p: point] returns i32:
    return $p.x;
end fn point.%x

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $p: point set_to point[x set_to 3];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($p.%x() x 2));
    return void;
end fn.main
