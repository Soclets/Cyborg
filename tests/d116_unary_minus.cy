// trace: D116
import std;

fn.main [none] -> void:
    let $x: i32 = 5;
    let $y: i32 = -$x;
    let $z: i32 = -7 + -$y x 2;
    let $f: f64.stable = 0.5;
    let $g: f64.stable = -$f;
    std::io::fmt_print("{}{newln}", $y);
    std::io::fmt_print("{}{newln}", $z);
    if $g: f64.stable[min: -0.5, max: -0.5]:
        std::io::fmt_print("negated{newln}");
    end if
    return void;
end fn.main
