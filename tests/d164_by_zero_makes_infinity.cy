// trace: D164, D158
import std;

fn.main [none] -> void:
    let $zero: f64.stable = 0.0;
    std::io::fmt_print("before{newln}");
    let $a: f64.stable = 1.0 div $zero;
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
