// trace: D133, D24
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $m: std::map.hash<key: str, val: i32> = empty;
    defer std::map.done(Give $m, alloc: Lend $heap);
    let $n: i32 = $m[pos: 1];
    return void;
end fn.main
