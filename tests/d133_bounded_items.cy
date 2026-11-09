// trace: D133, D20
import std;
@config(default);

fn.main [] -> void:
    let mut $a: std::array<i32[min: 0, max: 9]>[len: 1 items] = [1];
    let $big: i32 = 10;
    std::println("before");
    change $a[pos: 1] = $big;
    std::println("after");
    return void;
end fn.main
