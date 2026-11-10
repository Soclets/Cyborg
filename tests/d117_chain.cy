// trace: D117
import std;

fn.main [none] -> void:
    let $x: i32 = 3;
    if 1 < $x < 5:
        std::io::fmt_print("between{newln}");
    end if
    if 1 < $x < 2:
        std::io::fmt_print("not reached{newln}");
    end if
    if 1 < 2 at_most 2 == 2 != 3:
        std::io::fmt_print("any mix{newln}");
    end if
    return void;
end fn.main
