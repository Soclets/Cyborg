// The manifest's errors show first, then the program's.
// trace: D183, D207
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    let $a: f64.stable = 0.1;
    return void;
end fn.main
