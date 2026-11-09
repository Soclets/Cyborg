// trace: D12, D103
@config(default);
import std;

fn.main [] -> void:
    let $a: std::array<i32>[len: 3 items] = empty;
    return void;
end fn.main
