// trace: D96, D13
import std;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: 1 items] = empty;
    $xs.std::%append(val: 1) ?? void;
    $xs.std::%append(val: 2) ?? void;
    std::println($xs.len as str);
    return void;
end fn.main
