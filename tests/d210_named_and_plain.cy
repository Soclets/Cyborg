// trace: D210, D247
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $t: i32 = 21;
    std::io::fmt_print(Lend.mut $out, "{$t} {}{newln}", 3);
    return void;
end fn.main
