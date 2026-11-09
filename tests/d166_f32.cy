// trace: D166, D159
@config(default);
import std;

fn.main [] -> void:
    let $zero: f32.ieee = 0.0;
    let $inf: f32.ieee = 1.0 div $zero;
    std::io::fmt_print("{}{newln}", $inf);
    std::io::fmt_print("{}{newln}", ($zero div $zero));
    return void;
end fn.main
