// trace: D102, D103
import std;
@config(default);

fn.main [] -> void:
    let $b: std::list<i32>[len.min: 1 items, len.max: 8 items] = empty;
    return void;
end fn.main
