// A compiler may merge this, so nothing here prints the answer: how close counts isn't decided.
// trace: D163, D158
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.fast = 1.5;
    let $b: f64.fast = $a + $a x 2.0;
    std::io::fmt_print(Lend.mut $out, "ran{newln}");
    return void;
end fn.main
