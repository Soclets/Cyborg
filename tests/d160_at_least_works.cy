// trace: D160
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = 0.5;
    if $a at_least 0.25:
        std::io::fmt_print("at least{newln}");
    end if
    if $a at_most 0.25:
        std::io::fmt_print("at most{newln}");
    end if
    if $a < 0.75:
        std::io::fmt_print("below{newln}");
    end if
    if $a > 0.75:
        std::io::fmt_print("above{newln}");
    end if
    return void;
end fn.main
