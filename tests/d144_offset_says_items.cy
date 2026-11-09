// trace: D144, D143
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    std::println($xs[offset.items: 0] as str);
    std::println($xs[offset.items: 1 items] as str);
    let $n: u64.items = 2 items;
    std::println($xs[offset.items: $n] as str);
    let $i: i32 = 1;
    std::println($xs[offset.items: $i] as str);
    return void;
end fn.main
