// trace: D140, D7
import std;

fn.main [none] -> void:
    let $xs: std::list<i32>[len.max: inf items, len.type: u32] = empty;
    let $n: u64[unit: items] = $xs.len;
    return void;
end fn.main
