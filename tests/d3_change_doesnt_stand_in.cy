// trace: D3
import std;
import os;

fn.main [none] -> void:
    let mut $xs: std::coll::list<i32>[len.max: 8 items] = empty;
    change $xs = $xs.append(val: 4);
    return void;
end fn.main
