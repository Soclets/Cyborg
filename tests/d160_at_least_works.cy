// trace: D160
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable = 0.5;
    if $a at_least 0.25:
        std::io::fmt_print(Lend.mut $out, "at least{newln}");
    end if
    if $a at_most 0.25:
        std::io::fmt_print(Lend.mut $out, "at most{newln}");
    end if
    if $a < 0.75:
        std::io::fmt_print(Lend.mut $out, "below{newln}");
    end if
    if $a > 0.75:
        std::io::fmt_print(Lend.mut $out, "above{newln}");
    end if
    return void;
end fn.main
