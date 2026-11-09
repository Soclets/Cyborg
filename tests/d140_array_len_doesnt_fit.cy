// trace: D140
import std;
@config(default);

fn.main [] -> void:
    let $a: std::array<i32>[len: 300 items, len.type: u8] = all 0;
    return void;
end fn.main
