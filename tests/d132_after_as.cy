// trace: D132, D7
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = 5;
    let $s: str = $n as str ?? "none";
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn.main
