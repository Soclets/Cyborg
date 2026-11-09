// After a `%` the value is always still alive.
// trace: D5
@config(default);
import std;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::%%append(val: 1);
    $xs.std::%%append(val: 2);
    std::io::fmt_print("{}{newln}", $xs.len);
    return void;
end fn.main
