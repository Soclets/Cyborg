// trace: D128, D2
import std;

fn.main [none] returns void:
    let mut $xs: std::list<i32>[len.max: inf items] set_to [1, 2];
    for $x in $xs:
        change $xs set_to [3];
    end for
    return void;
end fn.main
