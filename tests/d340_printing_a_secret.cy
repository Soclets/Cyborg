// Printing a secret would make it public, so it's refused.
// trace: D340
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $key: u32.secret = 5;
    std::io::fmt_print(Lend.mut $out, "{$key}{newln}");
    return void;
end fn.main
