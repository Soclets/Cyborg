// A bare `val:` is always a value, so a map's types are `key.type:` and `val.type:`.
// trace: D366
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $m: std::coll::map.hash<key: str, val: i32> = empty;
    defer std::coll::map.done(Give $m, alloc: Lend $heap);
    return void;
end fn.main
