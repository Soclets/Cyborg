// trace: D124
import std;

fn.main [none] -> void:
    let $n: i32 = 0;
    if $n != 0 and 10 div.trunc $n > 1:
        std::io::fmt_print("not reached{newln}");
    end if
    std::io::fmt_print("safe{newln}");
    return void;
end fn.main
