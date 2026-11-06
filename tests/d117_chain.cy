// trace: D117
import std;

fn.main [] -> void:
    let $x: i32 = 3;
    if 1 < $x < 5:
        std::println("between");
    end if
    if 1 < $x < 2:
        std::println("not reached");
    end if
    if 1 < 2 at_most 2 == 2 != 3:
        std::println("any mix");
    end if
    return void;
end fn.main
