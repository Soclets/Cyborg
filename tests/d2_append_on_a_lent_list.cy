// trace: D4, D3
@config(default);
import std;

fn add [$xs: std::list<i32>[len.max: inf items]] -> void:
    $xs.std::%append(val: 1);
    return void;
end fn add

fn.main [] -> void:
    return void;
end fn.main
