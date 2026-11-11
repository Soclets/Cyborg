// trace: D159, D158
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $reading: f32.ieee = 0.5;
    let $clean: f32.stable = $reading as f32.stable;
    if $clean: f32.stable[min: 0.5, max: 0.5]:
        std::io::fmt_print(Lend.mut $out, "clean{newln}");
    end if
    return void;
end fn.main
