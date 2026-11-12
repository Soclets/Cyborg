// A map owns heap memory, so it's `must_Give`, and forgetting `std::map.done` doesn't compile.
// trace: D325, D331
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $m: std::map.hash<key: str, val: i32> = empty;
    return void;
end fn.main
