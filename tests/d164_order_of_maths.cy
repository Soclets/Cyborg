// trace: D164, D116
@config(default);
import std;

fn.main [] -> void:
    let $a: f64.stable = 1.0 + 4.0 div 2.0;
    std::io::fmt_print("{}{newln}", $a);
    let $b: f64.stable = (1.0 + 4.0) div 2.0;
    std::io::fmt_print("{}{newln}", $b);
    let $c: f64.stable = -4.0 div 2.0;
    std::io::fmt_print("{}{newln}", $c);
    return void;
end fn.main
