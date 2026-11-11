// trace: D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $f: str set_to "hi";
    std::io::fmt_print(Lend.mut $out, $f);
    return void;
end fn.main
