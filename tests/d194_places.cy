// trace: D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: f64.stable set_to 2.0;
    let $b: f64.stable set_to ~0.1;
    let $c: f64.stable set_to ~9.996;
    std::io::fmt_print(Lend.mut $out, "{places: 3} {places: 2} {places: 2} {places: 0}{newln}", $a, $b, $c, $a);
    return void;
end fn.main
