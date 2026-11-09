// trace: D101, D13
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: 8] = empty;
    return void;
end fn.main
