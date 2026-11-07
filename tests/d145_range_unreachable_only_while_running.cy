// Only running shows these numbers, and a range that can't reach its end runs zero rounds.
// trace: D145
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
