// trace: D133, D24
import std;

fn.main [] -> void:
    let $m: std::map<key: str, val: i32> = empty;
    let $n: i32 = $m[pos: 1];
    return void;
end fn.main
