// trace: D2
import std;
@config(default);

struct box:
    n: i32;
end struct box

fn replace [$b: Lent.mut box] -> void:
    change $b = box[n = 9];
    return void;
end fn replace

fn.main [] -> void:
    let mut $b: box = box[n = 1];
    replace(Lend.mut $b);
    std::io::fmt_print("{}{newln}", $b.n);
    return void;
end fn.main
