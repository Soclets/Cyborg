// `-0.0` and `0.0` stop in the same place, so these are the same type.
// trace: D160, D158
@config(default);
import std;

fn take [$x: f64[min: 0.0, max: 1.0]] -> void:
    std::io::fmt_print("taken{newln}");
    return void;
end fn take

fn.main [] -> void:
    let $t: f64[min: -0.0, max: 1.0] = 0.5;
    take($t);
    return void;
end fn.main
