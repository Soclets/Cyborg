// trace: D57, D2
import std;

struct box:
    n: i32;
end struct box

fn make [] -> box:
    return box[n = 4];
end fn make

fn.main [] -> void:
    std::println(make().n as str);
    return void;
end fn.main
