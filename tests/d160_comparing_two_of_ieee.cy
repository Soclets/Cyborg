// trace: D160, D159
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: f64.ieee set_to 1.5;
    let $b: f64.ieee set_to 2.5;
    if $a at_most $b:
        std::io::fmt_print(Lend.mut $out, "at most{newln}");
    end if
    return void;
end fn.main
