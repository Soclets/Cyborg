// trace: D145
import std;

fn.main [] -> void:
    let $start: i32 = 1;
    for $i in range[from: $start, to: 5, step_by: -1]:
        std::println("inside");
    end for
    std::println("after");
    return void;
end fn.main
