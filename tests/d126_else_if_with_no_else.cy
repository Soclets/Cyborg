// trace: D126
import std;
@config(default);

fn.main [] -> void:
    let $n: i32 = 0;
    if $n > 0:
        std::println("positive");
    else if $n < 0:
        std::println("negative");
    end if
    std::println("carried on");
    return void;
end fn.main
