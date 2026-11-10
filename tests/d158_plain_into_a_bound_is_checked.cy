// trace: D158, D20
@config(default);
import std;

fn.main [] -> void:
    let $x: f64.stable = 130.5;
    std::io::fmt_print("before{newln}");
    let $t: f64.stable[min: -40.0, max: 125.0] = $x;
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
