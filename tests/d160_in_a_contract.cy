// trace: D160, D29
@config(default);
import std;

fn half [$x: f64.stable] -> f64.stable:
    requires $x at_least 0.0;
    return $x x 0.5;
end fn half

fn.main [] -> void:
    let $h: f64.stable = half(1.5);
    std::io::fmt_print("ok{newln}");
    let $i: f64.stable = half(-1.5);
    return void;
end fn.main
