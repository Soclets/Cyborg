// `.` never runs code, so the short name still goes after `.%`.
// trace: D178, D8
import std;

use std::list.%append as %append;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.append(val: 4);
    return void;
end fn.main
