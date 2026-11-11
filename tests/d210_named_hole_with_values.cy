// trace: D210, D194
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $t: i32 = 21;
    std::io::fmt_print(Lend.mut $out, "temp: {$t} C{newln}", $t);
    return void;
end fn.main
