// trace: D2
import std;
import os;

struct box:
    n: i32;
end struct box

fn replace [$b: Lent.mut box] returns void:
    change $b set_to box[n set_to 9];
    return void;
end fn replace

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $b: box set_to box[n set_to 1];
    replace(Lend.mut $b);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.n);
    return void;
end fn.main
