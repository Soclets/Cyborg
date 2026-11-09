// trace: D180
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::array<i32>[len: 2 items] = [1, 2];
    for $n in Copy $xs:
        return void;
    end for
    return void;
end fn.main
