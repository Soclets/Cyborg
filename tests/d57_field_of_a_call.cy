// trace: D57, D2
import std;

struct box:
    n: i32;
end struct box

fn make [none] -> box:
    return box[n = 4];
end fn make

fn.main [none] -> void:
    std::io::fmt_print("{}{newln}", make().n);
    return void;
end fn.main
