// trace: D142, D26
import std;

fn.main [none] -> void:
    let $xs: std::list<i32>[len.max: 8 items] = empty;
    let $n: i32 = $xs[pos: $xs.len];
    return void;
end fn.main
