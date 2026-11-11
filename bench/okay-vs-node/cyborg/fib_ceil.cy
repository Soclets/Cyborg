import std;
import os;
use std::io::fmt_print as fmt_print;

// Ceil: as fast as Cyborg can go. fib(35) by walking up, 35 steps.
fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    let mut $a: i64 = 0;
    let mut $b: i64 = 1;
    let mut $i: i64 = 0;
    while $i below 35:
        let $next: i64 = $a + $b;
        change $a = $b;
        change $b = $next;
        change $i = $i + 1;
    end while
    fmt_print(Lend.mut $out, "{$a}{newln}");
    os::done(Give $out);
    return void;
end fn.main
