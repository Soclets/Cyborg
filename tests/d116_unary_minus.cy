// trace: D116
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $x: i32 set_to 5;
    let $y: i32 set_to -$x;
    let $z: i32 set_to -7 + -$y x 2;
    let $f: f64.stable set_to 0.5;
    let $g: f64.stable set_to -$f;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $y);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $z);
    if $g: f64.stable[min: -0.5, max: -0.5]:
        std::io::fmt_print(Lend.mut $out, "negated{newln}");
    end if
    return void;
end fn.main
