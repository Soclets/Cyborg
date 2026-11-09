// trace: D158, D20
@config(default);
import std;

fn.main [] -> void:
    let $n: f64[min: 1.0, max: 2.0] = 1.5;
    let $w: f64[min: 0.0, max: 3.0] = $n;
    std::io::fmt_print("fits{newln}");
    return void;
end fn.main
