// The check can't see these numbers, and nothing says what happens once they're known.
// trace: D128
import std;

fn.main [] -> void:
    let $start: i32 = 5;
    std::println("before");
    for $i in range[from: $start, to: 1]:
        std::println("inside");
    end for
    std::println("after");
    return void;
end fn.main
