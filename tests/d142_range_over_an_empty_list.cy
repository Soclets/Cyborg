// `from: 1` is past a length of 0, and what a range does when only running shows that isn't
// decided. Going through every position of an empty list is the common way to hit it.
// trace: D142, D128
import std;

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: 8 items] = empty;
    for $i in range[from: 1, to: $xs.len]:
        std::println("inside");
    end for
    std::println("after");
    return void;
end fn.main
