// A function declared with `[none]` is still called with `()`.
// trace: D267
import std;
import os;

fn seven [none] -> i32:
    return 7;
end fn seven

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = seven();
    std::io::fmt_print(Lend.mut $out, "{$n}{newln}");
    return void;
end fn.main
