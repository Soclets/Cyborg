// The short name is for `std::list`, so on an array it's still the bare, base-or-yours meaning.
// trace: D178, D115
import std;

use std::list.%%append as %%append;

fn.main [none] returns void:
    let mut $a: std::array<i32>[len: 2 items] set_to [1, 2];
    $a.%%append(val: 4);
    return void;
end fn.main
