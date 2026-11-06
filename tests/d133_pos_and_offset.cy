// trace: D133
import std;

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: 8 items] = [10, 20, 30];
    let $a: std::array<i32>[len: 2 items] = [7, 8];
    std::println($xs[pos: 1] as str);
    std::println($xs[offset: 0] as str);
    std::println($xs[pos: 3] as str);
    std::println($xs[offset: 2] as str);
    std::println($a[pos: 2] as str);
    let $i: i32 = 2;
    std::println($xs[pos: $i] as str);
    std::println($xs[offset: $i - 1] as str);
    let $w: i64 = 1;
    std::println($xs[pos: $w] as str);
    std::println(($xs[pos: 1] + $a[offset: 0]) as str);
    return void;
end fn.main
