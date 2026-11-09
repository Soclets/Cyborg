// trace: D133, D26
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: 8 items] = [10, 20];
    std::println("before");
    std::println($xs[pos: 3] as str);
    std::println("after");
    return void;
end fn.main
