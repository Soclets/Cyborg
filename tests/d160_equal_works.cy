// trace: D160
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: f64.stable set_to 0.5;
    if $a == 0.5:
        std::io::fmt_print(Lend.mut $out, "equal{newln}");
    end if
    if $a != 0.5:
        std::io::fmt_print(Lend.mut $out, "different{newln}");
    end if
    return void;
end fn.main
