// trace: D164, D162
import std;
@config(default);

fn.main [] -> void:
    let $one: f64 = 1.0;
    let $third: f64 = $one div 3.0;
    std::io::fmt_print("{}{newln}", $third);
    std::io::fmt_print.stored("{}{newln}", $third);
    let $small: f32 = 1.0;
    let $f: f32 = $small div 3.0;
    std::io::fmt_print("{}{newln}", $f);
    std::io::fmt_print.stored("{}{newln}", $f);
    return void;
end fn.main
