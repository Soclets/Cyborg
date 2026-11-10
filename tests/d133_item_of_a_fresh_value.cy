// trace: D133
import std;

fn make [none] -> std::array<i32>[len: 2 items]:
    return [4, 5];
end fn make

fn.main [none] -> void:
    std::io::fmt_print("{}{newln}", make()[pos: 2]);
    return void;
end fn.main
