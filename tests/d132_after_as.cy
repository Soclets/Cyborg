// trace: D132, D7
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $n: i32 set_to 5;
    let $s: str set_to $n as str ?? "none";
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn.main
