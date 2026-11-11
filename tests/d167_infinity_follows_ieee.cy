// trace: D167, D166
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $zero: f64.ieee = 0.0;
    let $inf: f64.ieee = 1.0 div $zero;
    let $down: f64.ieee = -1.0 div $zero;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($inf + 1.0));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($inf x -2.0));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($down - $inf));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($inf + $down));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (1.0 div $inf));
    if $down < $inf:
        std::io::fmt_print(Lend.mut $out, "ordered{newln}");
    end if
    return void;
end fn.main
