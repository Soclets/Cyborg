// An arena owns its block, so leaving without ending it doesn't compile.
// trace: D330, D331
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $arena: std::mem::arena = std::mem::arena.new(room: 64 kib, alloc: Lend $heap);
    return void;
end fn.main
