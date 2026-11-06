// trace: D20
import std;

fn.main [] -> void:
    let $n: i32 = 150;
    std::println("before");
    let $b: i32[min: 0, max: 100] = $n;
    std::println("after");
    return void;
end fn.main
