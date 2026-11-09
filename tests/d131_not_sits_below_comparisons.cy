// trace: D131
@config(default);
import std;

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = 2;
    if not $a < 3:
        std::io::fmt_print("not reached{newln}");
    else:
        std::io::fmt_print("a is under 3{newln}");
    end if
    if not $a == $b:
        std::io::fmt_print("they differ{newln}");
    end if
    if not $a + 1 == $b:
        std::io::fmt_print("not reached{newln}");
    else:
        std::io::fmt_print("the sum matches{newln}");
    end if
    if not not $a == 1:
        std::io::fmt_print("twice{newln}");
    end if
    return void;
end fn.main
