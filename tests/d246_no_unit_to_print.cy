// `unit` only applies to a value with a unit.
// trace: D246
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = 5;
    std::io::fmt_print(Lend.mut $out, "{$n, unit}{newln}");
    return void;
end fn.main
