// trace: D159
import std;

fn.main [none] -> void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    std::io::fmt_print("before{newln}");
    let $clean: f64.stable = $x as f64.stable;
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
