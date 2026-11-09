// trace: D133
import std;
@config(default);

fn make [] -> std::array<i32>[len: 2 items]:
    return [4, 5];
end fn make

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", make()[pos: 2]);
    return void;
end fn.main
