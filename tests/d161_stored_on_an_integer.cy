// trace: D161
import std;
@config(default);

fn.main [] -> void:
    let $n: i32 = 5;
    std::println.stored($n);
    return void;
end fn.main
