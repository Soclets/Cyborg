// trace: D210
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "temp: {$t} C{newln}");
    return void;
end fn.main
