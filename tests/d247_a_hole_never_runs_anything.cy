// A hole reads a value, never a call or maths.
// trace: D247
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $count: i32 = 3;
    std::io::fmt_print(Lend.mut $out, "{$count + 1}{newln}");
    return void;
end fn.main
