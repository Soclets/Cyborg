// trace: D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $n: i32 set_to -1;
    std::io::fmt_print(Lend.mut $out, "{base: hex}{newln}", $n);
    return void;
end fn.main
