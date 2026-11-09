// trace: D26
import std;
@config(default);

fn.main [] -> void:
    let $z: i32 = 0;
    std::println("before");
    let $a: i32 = 1 div.trunc $z;
    std::println("after");
    return void;
end fn.main
