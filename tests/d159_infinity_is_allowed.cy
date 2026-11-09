// trace: D159, D158
import std;
@config(default);

fn.main [] -> void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 12]:
        change $x = $x x $x;
    end for
    std::io::fmt_print("still running{newln}");
    return void;
end fn.main
