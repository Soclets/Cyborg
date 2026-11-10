// trace: D160
@config(default);
import std;

fn.main [] -> void:
    let $n: f64.stable = -0.0;
    let $p: f64.stable = 0.0;
    if $n == $p:
        std::io::fmt_print("equal{newln}");
    end if
    if $n != $p:
        std::io::fmt_print("different{newln}");
    end if
    if $n < $p:
        std::io::fmt_print("below{newln}");
    end if
    if $n at_least $p:
        std::io::fmt_print("at least{newln}");
    end if
    return void;
end fn.main
