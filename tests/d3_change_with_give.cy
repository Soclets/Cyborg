// trace: D3
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: str[len.max: 32 bytes] = "a";
    let mut $t: str[len.max: 32 bytes] = "b";
    change $t = Give $s;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $t);
    return void;
end fn.main
