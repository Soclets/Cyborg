// trace: D20
import std;
@config(default);

fn.main [] -> void:
    let $n: i32 = 150;
    std::io::fmt_print("before{newln}");
    let $b: i32[min: 0, max: 100] = $n;
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
