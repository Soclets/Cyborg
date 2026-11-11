// Arithmetic gives back a plain `i32`, and `change` checks it.
// trace: D20
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $b: i32[min: 0, max: 100] = 100;
    change $b = $b + 1;
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
