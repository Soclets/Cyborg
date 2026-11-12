// Every `std` name lives in a group, so the old ungrouped spelling is refused, with the group
// in the fix.
// trace: D378
import std;

fn.main [none] -> void:
    let $xs: std::list<i32>[len.max: 4 items] = [1, 2];
    return void;
end fn.main
