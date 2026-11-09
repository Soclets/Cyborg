// trace: D140, D7
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: inf items, len.type: u32] = empty;
    let $n: u64.items = $xs.len;
    return void;
end fn.main
