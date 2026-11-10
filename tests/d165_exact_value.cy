// trace: D165
@config(default);
import std;

fn.main [] -> void:
    let $a: f64.stable = exponent 1.5e3;
    let $b: f64.stable = exponent 0.0001e4;
    let $c: f64.stable = exponent 2.5e0;
    let $d: f64.stable = exponent -1.1e2;
    std::io::fmt_print("{}{newln}", $a);
    std::io::fmt_print("{}{newln}", $b);
    std::io::fmt_print("{}{newln}", $c);
    std::io::fmt_print("{}{newln}", $d);
    return void;
end fn.main
