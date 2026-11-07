// trace: D138, D7
import std;

fn.main [] -> void:
    let $h: std::map.hash<key: str, val: i32> = empty;
    let $o: std::map.ordered<key: str, val: i32> = Copy $h;
    return void;
end fn.main
