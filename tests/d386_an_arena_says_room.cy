// An arena's bookkeeping sits outside it, so it says `room:`, exactly the space for lists.
// trace: D386, D330
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $arena: std::mem::arena = std::mem::arena.new(size: 1 mib, alloc: Lend $heap);
    std::mem::arena.done(Give $arena, alloc: Lend $heap);
    return void;
end fn.main
