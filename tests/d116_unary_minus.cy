// trace: D116
import std;
@config(default);

fn.main [] -> void:
    let $x: i32 = 5;
    let $y: i32 = -$x;
    let $z: i32 = -7 + -$y x 2;
    let $f: f64 = 0.5;
    let $g: f64 = -$f;
    std::io::fmt_print("{}{newln}", $y);
    std::io::fmt_print("{}{newln}", $z);
    if $g: f64[min: -0.5, max: -0.5]:
        std::io::fmt_print("negated{newln}");
    end if
    return void;
end fn.main
