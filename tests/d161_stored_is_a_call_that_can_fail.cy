// trace: D161, D95
import std;
import os;

fn.pure p [$out: Lent os::stdout.unbuffered, $x: f64.stable] -> f64.stable:
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $x);
    return $x;
end fn.pure p

fn.main [none] -> void:
    return void;
end fn.main
