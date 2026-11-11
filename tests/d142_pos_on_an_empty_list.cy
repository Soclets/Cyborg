// trace: D142, D26
import std;

fn.main [none] returns void:
    let $xs: std::list<i32>[len.max: 8 items] set_to empty;
    let $n: i32 set_to $xs[pos: $xs.len];
    return void;
end fn.main
