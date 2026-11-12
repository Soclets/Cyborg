// trace: D130, D115
import std;

fn.main [none] -> void:
    let $xs: std::coll::array<i32>[len: 1 items] = [1];
    $xs.%total();
    return void;
end fn.main
