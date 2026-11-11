// trace: D214
import std;
import os;

fn my::main [none] -> i32:
    return 7;
end fn my::main

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", my::main());
    return void;
end fn.main
