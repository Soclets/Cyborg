// trace: D159, D158
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $reading: f64.ieee set_to 1.5;
    let $clean: f64.stable set_to $reading as f64.stable;
    if $clean: f64.stable[min: 1.5, max: 1.5]:
        std::io::fmt_print(Lend.mut $out, "clean{newln}");
    end if
    return void;
end fn.main
