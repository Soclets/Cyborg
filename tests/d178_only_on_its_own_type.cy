// The short name is for `std::coll::list`, so on an array it's still the bare, base-or-yours meaning.
// trace: D178, D115
import std;

use std::coll::list.%%append as %%append;

fn.main [none] -> void:
    let mut $a: std::coll::array<i32>[len: 2 items] = [1, 2];
    $a.%%append(val: 4);
    return void;
end fn.main
