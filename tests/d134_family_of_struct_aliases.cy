// trace: D134, D57
import std;
import os;

struct point:
    x: i32;
end struct point

fam.type.alias shapes:
    shapes.dot set_to point;
end fam.type.alias shapes

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $d: shapes.dot set_to shapes.dot[x set_to 4];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $d.x);
    return void;
end fn.main
