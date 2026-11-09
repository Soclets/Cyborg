// trace: D140, D100
import std;
@config(default);

fn.main [] -> void:
    let $a: std::array<i32>[len: 1 items] = [1];
    let $n: u64.items = $a.len x 2;
    return void;
end fn.main
