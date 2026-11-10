// trace: D194, D165
@config(default);
import std;

fn.main [] -> void:
    let $a: f64.stable = 1000.0;
    let $b: f64.stable = 12.5;
    let $c: f64.stable = ~99.99;
    std::io::fmt_print("{form: science} {form: science} {form: science, places: 1}{newln}", $a, $b, $c);
    return void;
end fn.main
