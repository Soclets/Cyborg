// trace: D144, D143
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    std::io::fmt_print("{}{newln}", $xs[offset.items: 0]);
    std::io::fmt_print("{}{newln}", $xs[offset.items: 1 items]);
    let $n: u64.items = 2 items;
    std::io::fmt_print("{}{newln}", $xs[offset.items: $n]);
    let $i: i32 = 1;
    std::io::fmt_print("{}{newln}", $xs[offset.items: $i]);
    return void;
end fn.main
