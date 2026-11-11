// trace: D140, D7
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 1 items] set_to [1];
    let $n: i32 set_to $a.len;
    return void;
end fn.main
