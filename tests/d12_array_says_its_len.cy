// trace: D12, D13
import std;
@config(default);

fn.main [] -> void:
    let $a: std::array<i32>[len.max: 3 items] = [1];
    return void;
end fn.main
