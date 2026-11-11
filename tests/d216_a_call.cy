// trace: D216
import std;
import os;

fn half [$n: i32] -> i32:
    return $n div.trunc 2;
end fn half

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $h := half(9);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $h);
    return void;
end fn.main
