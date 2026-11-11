// trace: D194
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable = 2.0;
    let $b: f64.stable = ~0.1;
    let $c: f64.stable = ~9.996;
    std::io::fmt_print(Lend.mut $out, "{places: 3} {places: 2} {places: 2} {places: 0}{newln}", $a, $b, $c, $a);
    return void;
end fn.main
