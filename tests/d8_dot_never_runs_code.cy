// trace: D8
import std;
@config(default);

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::append(val: 4);
    return void;
end fn.main
