// trace: D125, D20
@config(default);
import std;

struct box:
    n: i32[min: 0, max: 10];
end struct box

fn.main [] -> void:
    let mut $b: box = box[n = 10];
    std::io::fmt_print("before{newln}");
    change $b.n = $b.n + 1;
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
