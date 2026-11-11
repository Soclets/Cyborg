// trace: D3
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: str = "a";
    let mut $t: str = "b";
    change $t = Give $s;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $t);
    return void;
end fn.main
