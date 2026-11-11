import std;
import os;
use std::io::fmt_print as fmt_print;

fn.main [none] returns void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    let mut $sum: i64 = 0;
    let mut $i: i64 = 1;
    while $i at_most 100_000_000:
        change $sum = $sum + $i;
        change $i = $i + 1;
    end while
    fmt_print(Lend.mut $out, "{$sum}{newln}");
    os::done(Give $out);
    return void;
end fn.main
