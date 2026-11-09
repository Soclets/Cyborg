// trace: D194
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = 2.0;
    let $b: f64 = ~0.1;
    let $c: f64 = ~9.996;
    std::io::fmt_print("{places: 3} {places: 2} {places: 2} {places: 0}{newln}", $a, $b, $c, $a);
    return void;
end fn.main
