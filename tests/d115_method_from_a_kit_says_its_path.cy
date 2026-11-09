// trace: D115
@config(default);
import std;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%append(val: 4);
    return void;
end fn.main
