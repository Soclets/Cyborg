// trace: D12, D103
import std;
@config(default);

fn.main [] -> void:
    let $a: std::array<i32>[len: 3 items] = empty;
    return void;
end fn.main
