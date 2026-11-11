// trace: D158, D141
import std;
import os;

type.new ratio set_to f64.stable[min: 0.0, max: 1.0];

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $r: ratio set_to 0.5;
    let $x: f64.stable set_to $r as f64.stable;
    if $x: f64.stable[min: 0.5, max: 0.5]:
        std::io::fmt_print(Lend.mut $out, "crossed{newln}");
    end if
    return void;
end fn.main
