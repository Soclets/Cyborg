// trace: D3
import std;

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 1 items] set_to [1];
    for $x in $xs:
        change $x set_to 2;
    end for
    return void;
end fn.main
