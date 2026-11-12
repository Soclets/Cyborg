// A map owns heap memory, so it's `must_Give`, and forgetting `std::coll::map.done` doesn't compile.
// trace: D325, D331
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $m: std::coll::map.hash<key.type: str, val.type: i32> = empty;
    return void;
end fn.main
