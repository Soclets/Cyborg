// trace: D140, D7
import std;

fn.main [none] -> void:
    let $a: std::array<i32>[len: 1 items] = [1];
    let $n: i32[unit: items] = $a.len;
    return void;
end fn.main
