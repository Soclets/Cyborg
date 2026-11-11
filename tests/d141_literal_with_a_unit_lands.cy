// trace: D141, D101
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $t: u64[unit: items] = 5 items;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $t);
    return void;
end fn.main
