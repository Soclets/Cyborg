// trace: D141, D7
import std;

fn.main [none] -> void:
    let $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    let $n: i32 = $xs.len as i32;
    return void;
end fn.main
