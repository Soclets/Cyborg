// trace: D133, D26
import std;
@config(default);

fn.main [] -> void:
    let mut $a: std::array<i32>[len: 2 items] = [1, 2];
    change $a[pos: 3] = 9;
    std::println("after");
    return void;
end fn.main
