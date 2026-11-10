// trace: D140, D7
@config(default);
import std;

fn.main [] -> void:
    let $a: std::array<i32>[len: 1 items] = [1];
    let $n: i32[unit: items] = $a.len;
    return void;
end fn.main
