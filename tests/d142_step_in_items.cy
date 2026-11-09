// trace: D142, D128
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len: 5 items] = [1, 2, 3, 4, 5];
    for $i in range[from: 1, to: $xs.len, step_by: 2 items]:
        std::io::fmt_print("{}{newln}", $i);
    end for
    return void;
end fn.main
