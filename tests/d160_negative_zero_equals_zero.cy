// trace: D160
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: f64.stable = -0.0;
    let $p: f64.stable = 0.0;
    if $n == $p:
        std::io::fmt_print(Lend.mut $out, "equal{newln}");
    end if
    if $n != $p:
        std::io::fmt_print(Lend.mut $out, "different{newln}");
    end if
    if $n below $p:
        std::io::fmt_print(Lend.mut $out, "below{newln}");
    end if
    if $n at_least $p:
        std::io::fmt_print(Lend.mut $out, "at least{newln}");
    end if
    return void;
end fn.main
