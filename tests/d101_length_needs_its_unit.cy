// trace: D101, D13
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: 8] = empty;
    return void;
end fn.main
