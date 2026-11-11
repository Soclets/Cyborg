// trace: D133, D125
import std;

fn f [$a: std::array<i32>[len: 2 items]] returns void:
    change $a[pos: 1] = 9;
    return void;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
