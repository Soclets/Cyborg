// trace: D124
@config(default);
import std;

fn.main [] -> void:
    let $n: i32 = 0;
    if $n != 0 and 10 div.trunc $n > 1:
        std::io::fmt_print("not reached{newln}");
    end if
    std::io::fmt_print("safe{newln}");
    return void;
end fn.main
