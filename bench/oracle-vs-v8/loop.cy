import std;
import os;
use std::io::fmt_print as fmt_print;

fn.main [none] returns void:
    let mut $out: os::stdout.buffered set_to os::stdout.buffered(size: 1024 bytes);
    let mut $sum: i64 set_to 0;
    let mut $i: i64 set_to 1;
    while $i at_most 100_000_000:
        change $sum set_to $sum + $i;
        change $i set_to $i + 1;
    end while
    fmt_print(Lend.mut $out, "{$sum}{newln}");
    os::done(Give $out);
    return void;
end fn.main
