// trace: D124
import std;

fn.main [] -> void:
    let $n: i32 = 0;
    if $n == 0 or 10 div.trunc $n > 1:
        std::println("safe");
    end if
    return void;
end fn.main
