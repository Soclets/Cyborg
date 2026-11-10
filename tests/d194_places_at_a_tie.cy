// 0.125 is stored exactly, so two places is exactly halfway.
// trace: D194
@config(default);
import std;

fn.main [] -> void:
    let $x: f64.stable = 0.125;
    std::io::fmt_print("{places: 2}{newln}", $x);
    return void;
end fn.main
