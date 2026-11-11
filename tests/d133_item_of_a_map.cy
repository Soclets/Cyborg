// trace: D133, D24
import std;

fn.main [none] returns void:
    let $m: std::map.hash<key: str, val: i32> set_to empty;
    let $n: i32 set_to $m[pos: 1];
    return void;
end fn.main
