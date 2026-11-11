// trace: D164, D162
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $one: f64.stable set_to 1.0;
    let $third: f64.stable set_to $one div 3.0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $third);
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $third);
    let $small: f32.stable set_to 1.0;
    let $f: f32.stable set_to $small div 3.0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $f);
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $f);
    return void;
end fn.main
