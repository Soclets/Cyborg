// trace: D125, D20
import std;

struct box:
    n: i32[min: 0, max: 10];
end struct box

fn.main [] -> void:
    let mut $b: box = box[n = 10];
    std::println("before");
    change $b.n = $b.n + 1;
    std::println("after");
    return void;
end fn.main
