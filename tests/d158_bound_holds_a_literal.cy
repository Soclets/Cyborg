// trace: D158
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $t: f64.stable[min: -40.0, max: 125.0] set_to 21.5;
    let $x: f64.stable set_to $t;
    if $x: f64.stable[min: 21.5, max: 21.5]:
        std::io::fmt_print(Lend.mut $out, "held{newln}");
    end if
    return void;
end fn.main
