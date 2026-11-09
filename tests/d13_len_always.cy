// trace: D13
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len.always: 3 items] = [1, 2, 3];
    return void;
end fn.main
