// trace: D125, D20
import std;
import os;

struct box:
    n: i32[min: 0, max: 10];
end struct box

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $b: box = box[n = 10];
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    change $b.n = $b.n + 1;
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
