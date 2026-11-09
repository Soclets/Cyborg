// trace: D26
import std;
@config(default);

fn.main [] -> void:
    let $z: i32 = 0;
    std::io::fmt_print("before{newln}");
    let $a: i32 = 1 div.trunc $z;
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
