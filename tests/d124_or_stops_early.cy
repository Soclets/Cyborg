// trace: D124
@config(default);
import std;

fn.main [] -> void:
    let $n: i32 = 0;
    if $n == 0 or 10 div.trunc $n > 1:
        std::io::fmt_print("safe{newln}");
    end if
    return void;
end fn.main
