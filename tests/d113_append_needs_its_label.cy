// trace: D113, D114
import std;
@config(default);

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::%append(4);
    return void;
end fn.main
