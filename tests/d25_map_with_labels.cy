// trace: D24, D25
import std;
@config(default);

fn.main [] -> void:
    let $m: std::map.hash<key: str, val: i32> = empty;
    std::println("made");
    return void;
end fn.main
