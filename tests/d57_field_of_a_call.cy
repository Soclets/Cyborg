// trace: D57, D2
@config(default);
import std;

struct box:
    n: i32;
end struct box

fn make [] -> box:
    return box[n = 4];
end fn make

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", make().n);
    return void;
end fn.main
