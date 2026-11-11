// trace: D2
import std;
import os;

struct box:
    s: str;
end struct box

fn keep [$s: Given str] -> str:
    return Give $s;
end fn keep

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $b: box = box[s = "a"];
    let $k: str = keep(Copy $b.s);
    std::io::fmt_print(Lend.mut $out, "{}{}{newln}", $k, $b.s);
    return void;
end fn.main
