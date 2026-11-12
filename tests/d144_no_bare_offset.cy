// trace: D144, D143
import std;

fn.main [none] -> void:
    let $xs: std::coll::array<i32>[len: 3 items] = [4, 5, 6];
    let $n: i32 = $xs[offset: 1];
    return void;
end fn.main
