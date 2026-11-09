// An `else` can't reach past the `for` around it to an `if` outside.
// trace: D126
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::array<i32>[len: 1 items] = [1];
    if 1 < 2:
        for $x in $xs:
            std::println("x");
        else:
            std::println("y");
        end for
    end if
    return void;
end fn.main
