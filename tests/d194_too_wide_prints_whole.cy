// trace: D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "[{align: right, width: 3 cols}]{newln}", "longer");
    return void;
end fn.main
