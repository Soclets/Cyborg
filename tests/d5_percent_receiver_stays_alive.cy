// After a `%` the value is always still alive.
// trace: D5
import std;
@config(default);

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::%append(val: 1);
    $xs.std::%append(val: 2);
    std::println($xs.len as str);
    return void;
end fn.main
