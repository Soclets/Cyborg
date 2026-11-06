// trace: D124
import std;

fn.main [] -> void:
    let $n: i32 = 0;
    if $n != 0 and 10 div.trunc $n > 1:
        std::println("not reached");
    end if
    std::println("safe");
    return void;
end fn.main
