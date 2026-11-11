// trace: D138, D7
import std;

fn.main [none] returns void:
    let $h: std::map.hash<key: str, val: i32> set_to empty;
    let $o: std::map.ordered<key: str, val: i32> set_to Copy $h;
    return void;
end fn.main
