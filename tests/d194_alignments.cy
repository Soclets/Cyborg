// trace: D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "[{align: left, width: 6 cols}]{newln}", "ab");
    std::io::fmt_print(Lend.mut $out, "[{align: right, width: 6 cols}]{newln}", "ab");
    std::io::fmt_print(Lend.mut $out, "[{align: center, width: 6 cols}]{newln}", "ab");
    return void;
end fn.main
