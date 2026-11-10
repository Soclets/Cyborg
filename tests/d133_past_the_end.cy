// trace: D133, D26
import std;

fn.main [none] -> void:
    let $xs: std::list<i32>[len.max: 8 items] = [10, 20];
    std::io::fmt_print("before{newln}");
    std::io::fmt_print("{}{newln}", $xs[pos: 3]);
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
