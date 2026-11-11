// trace: D180
import std;

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 2 items] set_to [1, 2];
    for $n in Copy $xs:
        return void;
    end for
    return void;
end fn.main
