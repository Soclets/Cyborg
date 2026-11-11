// trace: D158, D59
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $x: f32.stable set_to ~0.1;
    if $x: f32.stable[min: ~0.1, max: ~0.1]:
        std::io::fmt_print(Lend.mut $out, "the same rounding{newln}");
    end if
    return void;
end fn.main
