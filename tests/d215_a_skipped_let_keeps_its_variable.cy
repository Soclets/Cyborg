// A skipped let still creates its variable, with no value.
// trace: D215, D203
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $sum: i32 set_to totl(5);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $sum);
    return void;
end fn.main
