// trace: D165
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: f64.stable set_to exponent 1.5e3;
    let $b: f64.stable set_to exponent 0.0001e4;
    let $c: f64.stable set_to exponent 2.5e0;
    let $d: f64.stable set_to exponent -1.1e2;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $d);
    return void;
end fn.main
