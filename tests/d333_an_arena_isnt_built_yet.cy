// An arena is decided, and the oracle doesn't handle it yet.
// trace: D330, D333, D286
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $arena: std::arena = std::arena.new(size: 1 mib, alloc: Lend $heap);
    return void;
end fn.main
