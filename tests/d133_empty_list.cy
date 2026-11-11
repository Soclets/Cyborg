// trace: D133, D26
import std;

fn.main [none] returns void:
    let $xs: std::list<i32>[len.max: 8 items] set_to empty;
    let $n: i32 set_to $xs[pos: 1];
    return void;
end fn.main
