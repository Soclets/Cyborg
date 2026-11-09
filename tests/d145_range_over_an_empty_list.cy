// `from: 1` is past a length of 0, so going through an empty list just skips the loop.
// trace: D145, D142
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: 8 items] = empty;
    for $i in range[from: 1, to: $xs.len]:
        std::println("inside");
    end for
    std::println("after");
    return void;
end fn.main
