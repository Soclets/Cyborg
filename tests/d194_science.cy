// trace: D194, D165
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: f64.stable set_to 1000.0;
    let $b: f64.stable set_to 12.5;
    let $c: f64.stable set_to ~99.99;
    std::io::fmt_print(Lend.mut $out, "{form: science} {form: science} {form: science, places: 1}{newln}", $a, $b, $c);
    return void;
end fn.main
