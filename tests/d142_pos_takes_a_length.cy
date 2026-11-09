// trace: D142, D143
import std;
@config(default);

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: 8 items] = [10, 20, 30];
    std::println($xs[pos: $xs.len] as str);
    std::println($xs[pos: 3] as str);
    std::println($xs[pos: 3 items] as str);
    change $xs[pos: $xs.len] = 99;
    std::println($xs[pos: 3] as str);
    let $i: u64.items = 2 items;
    std::println($xs[pos: $i] as str);
    return void;
end fn.main
