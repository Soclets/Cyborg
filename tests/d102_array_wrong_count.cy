// trace: D102
import std;
@config(default);

fn.main [] -> void:
    let $a: std::array<i32>[len: 3 items] = [1, 2];
    return void;
end fn.main
