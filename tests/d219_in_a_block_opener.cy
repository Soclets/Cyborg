// A block opener stays on one line.
// trace: D219
@config(default);
import std;

fn.main [] -> void:
    let $a: i32 = 1;
    if
        $a < 3:
        std::io::fmt_print("small{newln}");
    end if
    return void;
end fn.main
