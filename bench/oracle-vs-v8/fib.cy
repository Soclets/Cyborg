import std;
import os;
use std::io::fmt_print as fmt_print;

fn fib [$n: i64] returns i64:
    if $n < 2:
        return $n;
    end if
    return fib($n - 1) + fib($n - 2);
end fn fib

fn.main [none] returns void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    let $r: i64 = fib(35);
    fmt_print(Lend.mut $out, "{$r}{newln}");
    os::done(Give $out);
    return void;
end fn.main
