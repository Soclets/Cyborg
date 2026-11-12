import std;
import os;
use std::io::fmt_print as fmt_print;

// Ceil: as fast as Cyborg can go. The sum of 1 to n is n x (n + 1) / 2.
fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    let $n: i64 = 100_000_000;
    let $sum: i64 = $n x ($n + 1) div.exact 2;
    fmt_print(Lend.mut $out, "{$sum}{newln}");
    os::done(Give $out);
    return void;
end fn.main
