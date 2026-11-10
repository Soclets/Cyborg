// trace: D160, D158
@config(default);
import std;

fn.main [] -> void:
    let $t: f64.stable[min: 0.0, max: 1.0] = -0.0;
    std::io::fmt_print("fits{newln}");
    return void;
end fn.main
