// trace: D2
import std;
import os;

struct box:
    s: str;
    t: str;
end struct box

fn replace [$s: Lent.mut str] returns void:
    change $s set_to "changed";
    return void;
end fn replace

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $b: box set_to box[s set_to "a", t set_to "b"];
    replace(Lend.mut $b.t);
    std::io::fmt_print(Lend.mut $out, "{}{}{newln}", $b.s, $b.t);
    return void;
end fn.main
