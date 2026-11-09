// trace: D164, D158
@config(default);
import std;

fn.main [] -> void:
    let $zero: f64 = 0.0;
    std::io::fmt_print("before{newln}");
    let $a: f64 = 1.0 div $zero;
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
