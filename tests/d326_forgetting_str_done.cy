// Heap text is `must_Give`, so forgetting `str.done` doesn't compile.
// trace: D326, D331
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $log: str[len.max: inf bytes] = str.from(5, alloc: Lend $heap);
    return void;
end fn.main
