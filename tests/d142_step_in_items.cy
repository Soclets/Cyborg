// trace: D142, D128
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::array<i32>[len: 5 items] = [1, 2, 3, 4, 5];
    for $i in range[from: 1, to: $xs.len, step_by: 2 items]:
        std::println($i as str);
    end for
    return void;
end fn.main
