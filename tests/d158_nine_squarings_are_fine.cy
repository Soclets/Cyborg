// trace: D158
@config(default);
import std;

fn.main [] -> void:
    let mut $x: f64 = 2.0;
    for $i in range[from: 1, to: 9]:
        change $x = $x x $x;
    end for
    std::io::fmt_print("2^512 is fine{newln}");
    return void;
end fn.main
