// trace: D126
import std;

fn.main [none] -> void:
    let $a: i32 = 1;
    let $b: i32 = 2;
    if $a > 5:
        std::io::fmt_print("no{newln}");
    else:
        if $b > 5:
            std::io::fmt_print("no{newln}");
        else if $b == 2:
            std::io::fmt_print("inner{newln}");
        end if
        std::io::fmt_print("after inner{newln}");
    end if
    return void;
end fn.main
