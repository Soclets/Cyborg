// trace: D159, D158
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $reading: f64.ieee = 1.5;
    let $clean: f64.stable = $reading as f64.stable;
    if $clean: f64.stable[min: 1.5, max: 1.5]:
        std::io::fmt_print(Lend.mut $out, "clean{newln}");
    end if
    return void;
end fn.main
