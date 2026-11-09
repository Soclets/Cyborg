// trace: D158, D20
import std;
@config(default);

fn.main [] -> void:
    let $t: f64[min: 0.0, max: 1.0] = 0.5;
    let $x: f64 = $t;
    if $x: f64[min: 0.5, max: 0.5]:
        std::io::fmt_print("same{newln}");
    end if
    return void;
end fn.main
