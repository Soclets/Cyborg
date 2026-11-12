// trace: D138, D7
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $h: std::coll::map.hash<key: str, val: i32> = empty;
    let $o: std::coll::map.ordered<key: str, val: i32> = Give $h;
    std::coll::map.done(Give $h, alloc: Lend $heap);
    return void;
end fn.main
