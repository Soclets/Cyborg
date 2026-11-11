// trace: D140, D101
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 1 items] set_to [1];
    let $n: u64 set_to $a.len;
    return void;
end fn.main
