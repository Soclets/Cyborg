import std;
import os;
use std::io::fmt_print as fmt_print;

// Purpose: square root and float division, n-body's inner cost, alone. 20,000,000 rounds of
// dt over (d2 times its square root), with d2 growing a little each round so nothing repeats,
// added up in order.

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    let mut $d2: f64.stable = 1.0;
    let mut $total: f64.stable = 0.0;
    let $dt: f64.stable = ~0.01;
    let $grow: f64.stable = ~0.000001;
    for $round in range[from: 1, to: 20_000_000]:
        change $total = $total + $dt div ($d2 x std::math::sqrt($d2));
        change $d2 = $d2 + $grow;
    end for
    fmt_print(Lend.mut $out, "{$total, places: 9}{newln}");
    os::done(Give $out);
    return void;
end fn.main
