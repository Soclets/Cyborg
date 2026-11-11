// trace: D180, D2
import std;

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 3 items] set_to [1, 2, 3];
    for $n in Lend.mut $xs:
        change $n set_to 0;
    end for
    return void;
end fn.main
