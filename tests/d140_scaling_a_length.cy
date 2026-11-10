// trace: D140, D100
@config(default);
import std;

fn.main [] -> void:
    let $a: std::array<i32>[len: 1 items] = [1];
    let $n: u64[unit: items] = $a.len x 2;
    return void;
end fn.main
