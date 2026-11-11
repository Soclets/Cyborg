// trace: D140, D7
import std;

fn.main [none] returns void:
    let $xs: std::list<i32>[len.max: inf items, len.type: u32] set_to empty;
    let $n: u64[unit: items] set_to $xs.len;
    return void;
end fn.main
