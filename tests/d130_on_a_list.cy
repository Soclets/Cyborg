// trace: D130, D115
import std;

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 1 items] = [1];
    $xs.%total();
    return void;
end fn.main
