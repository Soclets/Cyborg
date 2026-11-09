// trace: D142, D128
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    for $i in range[from: 1, to: $xs.len]:
        std::io::fmt_print("{} holds {}{newln}", $i, $xs[pos: $i]);
    end for
    for $i in range[from: 1 items, before: $xs.len]:
        std::io::fmt_print("{}{newln}", $i);
    end for
    for $i in range[from: 1 items, to: 2 items]:
        let $kept: i32.items = $i;
        std::io::fmt_print("{}{newln}", $kept);
    end for
    return void;
end fn.main
