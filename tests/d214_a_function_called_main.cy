// trace: D214
import std;
import os;

fn my::main [none] returns i32:
    return 7;
end fn my::main

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", my::main());
    return void;
end fn.main
