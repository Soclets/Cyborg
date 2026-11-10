// trace: D158
import std;

fn.main [none] -> void:
    let $t: f64.stable[min: -40.0, max: 125.0] = 21.5;
    let $x: f64.stable = $t;
    if $x: f64.stable[min: 21.5, max: 21.5]:
        std::io::fmt_print("held{newln}");
    end if
    return void;
end fn.main
