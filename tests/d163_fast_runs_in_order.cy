// A compiler may merge this, so nothing here prints the answer: how close counts isn't decided.
// trace: D163, D158
@config(default);
import std;

fn.main [] -> void:
    let $a: f64.fast = 1.5;
    let $b: f64.fast = $a + $a x 2.0;
    std::io::fmt_print("ran{newln}");
    return void;
end fn.main
