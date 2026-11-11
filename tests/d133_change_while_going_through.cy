// trace: D133, D2
import std;

fn.main [none] returns void:
    let mut $a: std::array<i32>[len: 2 items] set_to [1, 2];
    for $x in $a:
        change $a[pos: 1] set_to $x;
    end for
    return void;
end fn.main
