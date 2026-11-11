// trace: D160
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f32.stable = ~0.1;
    let $b: f32.stable = ~0.1;
    if $a == $b:
        std::io::fmt_print(Lend.mut $out, "the same rounding{newln}");
    end if
    return void;
end fn.main
