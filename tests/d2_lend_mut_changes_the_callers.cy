// trace: D2
import std;
import os;

struct box:
    n: i32;
end struct box

fn replace [$b: Lent.mut box] -> void:
    change $b = box[n = 9];
    return void;
end fn replace

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $b: box = box[n = 1];
    replace(Lend.mut $b);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.n);
    return void;
end fn.main
