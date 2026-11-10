// An `else` can't reach past the `for` around it to an `if` outside.
// trace: D126
import std;

fn.main [none] -> void:
    let $xs: std::array<i32>[len: 1 items] = [1];
    if 1 < 2:
        for $x in $xs:
            std::io::fmt_print("x{newln}");
        else:
            std::io::fmt_print("y{newln}");
        end for
    end if
    return void;
end fn.main
