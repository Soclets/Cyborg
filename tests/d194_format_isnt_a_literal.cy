// trace: D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $f: str = "hi";
    std::io::fmt_print(Lend.mut $out, $f);
    return void;
end fn.main
