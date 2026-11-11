// trace: D161, D160
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $z: f64.stable set_to -0.0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $z);
    return void;
end fn.main
