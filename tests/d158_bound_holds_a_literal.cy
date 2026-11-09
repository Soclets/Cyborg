// trace: D158
@config(default);
import std;

fn.main [] -> void:
    let $t: f64[min: -40.0, max: 125.0] = 21.5;
    let $x: f64 = $t;
    if $x: f64[min: 21.5, max: 21.5]:
        std::io::fmt_print("held{newln}");
    end if
    return void;
end fn.main
