// trace: D102
import std;
@config(default);

fn.main [] -> void:
    let $b: std::list<i32>[len.max: 1 items] = [4, 6];
    return void;
end fn.main
