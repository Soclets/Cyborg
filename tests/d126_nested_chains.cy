// trace: D126
import std;
@config(default);

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = 2;
    if $a > 5:
        std::println("no");
    else:
        if $b > 5:
            std::println("no");
        else if $b == 2:
            std::println("inner");
        end if
        std::println("after inner");
    end if
    return void;
end fn.main
