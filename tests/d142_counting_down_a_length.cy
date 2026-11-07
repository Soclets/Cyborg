// A length is unsigned, so its own type can't hold the step.
// trace: D142, D140
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    for $i in range[from: $xs.len, to: 1, step_by: -1]:
        return void;
    end for
    return void;
end fn.main
