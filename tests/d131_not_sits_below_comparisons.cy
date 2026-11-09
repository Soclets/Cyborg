// trace: D131
import std;
@config(default);

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = 2;
    if not $a < 3:
        std::println("not reached");
    else:
        std::println("a is under 3");
    end if
    if not $a == $b:
        std::println("they differ");
    end if
    if not $a + 1 == $b:
        std::println("not reached");
    else:
        std::println("the sum matches");
    end if
    if not not $a == 1:
        std::println("twice");
    end if
    return void;
end fn.main
