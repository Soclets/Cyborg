// A step of 0 never ends, so it's a failure, and a `for` has nothing to handle it.
// trace: D145
import std;
@config(default);

fn.main [] -> void:
    let $by: i32 = 0;
    std::println("before");
    for $i in range[from: 1, to: 5, step_by: $by]:
        std::println("inside");
    end for
    std::println("after");
    return void;
end fn.main
