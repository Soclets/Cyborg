// trace: D133, D125
import std;

fn f [$a: std::array<i32>[len: 2 items]] -> void:
    change $a[pos: 1] = 9;
    return void;
end fn f

fn.main [] -> void:
    return void;
end fn.main
