// trace: D133, D123
import std;

fn f [$n: i32] -> void:
    return void;
end fn f

fn.main [none] -> void:
    let $a: std::array<i32>[len: 1 items] = [1];
    f(Lend $a[pos: 1]);
    return void;
end fn.main
