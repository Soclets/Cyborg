// Which item isn't known, so nothing says these don't touch.
// trace: D133, D2
import std;

fn.main [none] returns void:
    let mut $g: std::list<std::list<i32>[len.max: 2 items]>[len.max: 2 items] set_to [[1], [2]];
    for $x in $g[pos: 1]:
        change $g[pos: 2] set_to [3];
    end for
    return void;
end fn.main
