// trace: D102
import std;
@config(default);

fn.main [] -> void:
    let $b: std::list<i32>[len.min: 3 items, len.max: 8 items] = [4, 6];
    return void;
end fn.main
