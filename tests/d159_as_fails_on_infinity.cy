// trace: D159
@config(default);
import std;

fn.main [] -> void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    std::io::fmt_print("before{newln}");
    let $clean: f64 = $x as f64;
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
