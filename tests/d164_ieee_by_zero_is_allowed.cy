// trace: D164, D159
import std;

fn.main [none] -> void:
    let $zero: f64.ieee = 0.0;
    let $inf: f64.ieee = 1.0 div $zero;
    let $nan: f64.ieee = $zero div $zero;
    std::io::fmt_print("still running{newln}");
    let $clean: f64.stable = $nan as f64.stable;
    return void;
end fn.main
