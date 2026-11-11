// trace: D149
import std;
import os;

type.new title set_to str;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $s: str set_to "a";
    let $t: title set_to Copy $s as title;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn.main
