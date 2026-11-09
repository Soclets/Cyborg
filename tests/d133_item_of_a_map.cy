// trace: D133, D24
import std;
@config(default);

fn.main [] -> void:
    let $m: std::map.hash<key: str, val: i32> = empty;
    let $n: i32 = $m[pos: 1];
    return void;
end fn.main
