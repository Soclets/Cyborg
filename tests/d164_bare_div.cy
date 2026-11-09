// trace: D164
@config(default);
import std;

fn.main [] -> void:
    let $a: f64 = 7.0 div 2.0;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
