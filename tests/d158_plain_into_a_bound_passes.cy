// trace: D158, D20
import std;
@config(default);

fn.main [] -> void:
    let $x: f64 = 20.5;
    let $t: f64[min: -40.0, max: 125.0] = $x;
    std::io::fmt_print("fits{newln}");
    return void;
end fn.main
