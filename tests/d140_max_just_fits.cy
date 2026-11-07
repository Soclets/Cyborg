// trace: D140
import std;

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: 255 items, len.type: u8] = empty;
    std::println($xs.len as str);
    return void;
end fn.main
