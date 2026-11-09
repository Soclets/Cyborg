// trace: D130, D115
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len: 1 items] = [1];
    $xs.%total();
    return void;
end fn.main
