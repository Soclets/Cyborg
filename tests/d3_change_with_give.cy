// trace: D3
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $s: str set_to "a";
    let mut $t: str set_to "b";
    change $t set_to Give $s;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $t);
    return void;
end fn.main
