// What the failure is called when a list is full isn't decided.
// trace: D96
@config(default);
import std;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: 1 items] = empty;
    $xs.std::%append(val: 1) otherwise:
        case too_long:
            return void;
    end otherwise
    return void;
end fn.main
