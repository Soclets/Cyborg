// trace: D2
import std;
import os;

struct box:
    s: str;
    t: str;
end struct box

fn replace [$s: Lent.mut str] -> void:
    change $s = "changed";
    return void;
end fn replace

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $b: box = box[s = "a", t = "b"];
    replace(Lend.mut $b.t);
    std::io::fmt_print(Lend.mut $out, "{}{}{newln}", $b.s, $b.t);
    return void;
end fn.main
