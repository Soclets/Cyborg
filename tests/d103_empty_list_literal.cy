// trace: D103
import std;
@config(default);

fn.main [] -> void:
    let $b: std::list<i32>[len.max: 8 items] = [];
    return void;
end fn.main
