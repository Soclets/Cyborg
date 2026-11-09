// trace: D140
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: 255 items, len.type: u8] = empty;
    std::io::fmt_print("{}{newln}", $xs.len);
    return void;
end fn.main
