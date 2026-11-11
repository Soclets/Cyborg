// trace: D57, D2
import std;
import os;

struct box:
    n: i32;
end struct box

fn make [none] returns box:
    return box[n = 4];
end fn make

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", make().n);
    return void;
end fn.main
