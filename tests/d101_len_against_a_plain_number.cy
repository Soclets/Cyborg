// trace: D101
import std;

fn.main [none] -> void:
    let $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    if $xs.len at_least 3:
        return void;
    end if
    return void;
end fn.main
