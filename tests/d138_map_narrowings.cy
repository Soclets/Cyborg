// trace: D138, D24, D25
import std;
@config(default);

fn.main [] -> void:
    let $h: std::map.hash<key: str, val: i32> = empty;
    let $o: std::map.ordered<key: str, val: i32> = empty;
    std::println("made");
    return void;
end fn.main
