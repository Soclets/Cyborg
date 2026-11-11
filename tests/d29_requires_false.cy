// trace: D29, D84
import std;
import os;

fn half [$n: i32] returns i32:
    requires $n at_least 0;

    return $n div.trunc 2;
end fn half

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", half(4));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", half(-4));
    return void;
end fn.main
