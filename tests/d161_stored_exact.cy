// Everything here fits in the digits the standard shows.
// trace: D161
@config(default);
import std;

fn.main [] -> void:
    let $a: f64 = 0.5;
    let $b: f64 = 1.0;
    let $c: f64 = -0.25;
    let $d: f32 = ~0.1;
    std::io::fmt_print.stored("{}{newln}", $a);
    std::io::fmt_print.stored("{}{newln}", $b);
    std::io::fmt_print.stored("{}{newln}", $c);
    std::io::fmt_print.stored("{}{newln}", $d);
    return void;
end fn.main
