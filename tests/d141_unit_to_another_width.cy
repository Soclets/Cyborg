// trace: D141, D7
import std;

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 3 items] set_to [1, 2, 3];
    let $n: i32 set_to $xs.len as i32;
    return void;
end fn.main
