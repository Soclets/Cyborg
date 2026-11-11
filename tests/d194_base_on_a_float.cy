// trace: D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $x: f64.stable set_to 2.5;
    std::io::fmt_print(Lend.mut $out, "{base: hex}{newln}", $x);
    return void;
end fn.main
