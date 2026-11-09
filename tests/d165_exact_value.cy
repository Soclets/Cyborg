// trace: D165
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = exponent 1.5e3;
    let $b: f64 = exponent 0.0001e4;
    let $c: f64 = exponent 2.5e0;
    let $d: f64 = exponent -1.1e2;
    std::io::fmt_print("{}{newln}", $a);
    std::io::fmt_print("{}{newln}", $b);
    std::io::fmt_print("{}{newln}", $c);
    std::io::fmt_print("{}{newln}", $d);
    return void;
end fn.main
