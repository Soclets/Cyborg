// trace: D140
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: 300 items, len.type: u8] = empty;
    return void;
end fn.main
