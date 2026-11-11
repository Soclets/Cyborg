// trace: D149
import std;
import os;

type.new title = str;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: str = "a";
    let $t: title = Copy $s as title;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn.main
