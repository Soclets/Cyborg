// trace: D140, D100
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 1 items] set_to [1];
    let $n: u64[unit: items] set_to $a.len x 2;
    return void;
end fn.main
