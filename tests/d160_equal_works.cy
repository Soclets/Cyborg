// trace: D160
@config(default);
import std;

fn.main [] -> void:
    let $a: f64 = 0.5;
    if $a == 0.5:
        std::io::fmt_print("equal{newln}");
    end if
    if $a != 0.5:
        std::io::fmt_print("different{newln}");
    end if
    return void;
end fn.main
