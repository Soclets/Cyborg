// trace: D102
@config(default);
import std;

fn.main [] -> void:
    let $d: std::list<u8>[len.max: 3 items] = all 0;
    return void;
end fn.main
