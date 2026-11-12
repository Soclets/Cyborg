import std;
import os;
use std::io::fmt_print as fmt_print;

fn fib [$n: i64] -> i64:
    if $n below 2:
        return $n;
    end if
    return fib($n - 1) + fib($n - 2);
end fn fib

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    let $r: i64 = fib(35);
    fmt_print(Lend.mut $out, "{$r}{newln}");
    os::done(Give $out);
    return void;
end fn.main
