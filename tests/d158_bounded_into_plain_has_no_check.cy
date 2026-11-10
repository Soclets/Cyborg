// trace: D158, D20
import std;

fn.main [none] -> void:
    let $t: f64.stable[min: 0.0, max: 1.0] = 0.5;
    let $x: f64.stable = $t;
    if $x: f64.stable[min: 0.5, max: 0.5]:
        std::io::fmt_print("same{newln}");
    end if
    return void;
end fn.main
