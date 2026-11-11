// trace: D160, D29
import std;
import os;

fn half [$x: f64.stable] returns f64.stable:
    requires $x at_least 0.0;
    return $x x 0.5;
end fn half

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $h: f64.stable = half(1.5);
    std::io::fmt_print(Lend.mut $out, "ok{newln}");
    let $i: f64.stable = half(-1.5);
    return void;
end fn.main
