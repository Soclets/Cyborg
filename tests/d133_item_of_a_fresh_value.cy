// trace: D133
import std;

fn make [] -> std::array<i32>[len: 2 items]:
    return [4, 5];
end fn make

fn.main [] -> void:
    std::println(make()[pos: 2] as str);
    return void;
end fn.main
