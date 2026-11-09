// trace: D8
@config(default);
import std;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::append(val: 4);
    return void;
end fn.main
