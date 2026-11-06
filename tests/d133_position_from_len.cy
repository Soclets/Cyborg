// trace: D133, D122
import std;

fn.main [] -> void:
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    let $n: i32 = $a[pos: $a.len];
    return void;
end fn.main
