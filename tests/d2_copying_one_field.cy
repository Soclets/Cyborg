// trace: D2
import std;
import os;

struct box:
    s: str[len.max: 32 bytes];
end struct box

fn keep [$s: Given str[len.max: 32 bytes]] -> str[len.max: 32 bytes]:
    return Give $s;
end fn keep

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $b: box = box[s = "a"];
    let $k: str[len.max: 32 bytes] = keep(Copy $b.s);
    std::io::fmt_print(Lend.mut $out, "{}{}{newln}", $k, $b.s);
    return void;
end fn.main
