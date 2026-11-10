// trace: D140
import std;

fn.main [none] -> void:
    let $xs: std::list<i32>[len.max: 255 items, len.type: u8] = empty;
    std::io::fmt_print("{}{newln}", $xs.len);
    return void;
end fn.main
