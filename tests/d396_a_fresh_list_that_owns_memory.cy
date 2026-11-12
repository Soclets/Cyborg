// A fresh list that owns memory has to end with `done`, which a bare loop can't do, so it's
// refused, and the fix stores it first.
// trace: D396, D325
import std;
import os;

fn.main [none] -> void:
    let mut $heap: os::heap = os::heap();
    for $x in std::coll::list.from([1, 2], alloc: Lend $heap):
    end for
    return void;
end fn.main
