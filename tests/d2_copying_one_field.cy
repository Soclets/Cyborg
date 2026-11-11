// trace: D2
import std;
import os;

struct box:
    s: str;
end struct box

fn keep [$s: Given str] returns str:
    return Give $s;
end fn keep

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $b: box set_to box[s set_to "a"];
    let $k: str set_to keep(Copy $b.s);
    std::io::fmt_print(Lend.mut $out, "{}{}{newln}", $k, $b.s);
    return void;
end fn.main
