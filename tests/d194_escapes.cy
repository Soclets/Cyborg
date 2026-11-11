// trace: D194, D197
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "\\ \" \' \{x\} \x41\u00e9\U0001F600{tab}|{newln}");
    return void;
end fn.main
