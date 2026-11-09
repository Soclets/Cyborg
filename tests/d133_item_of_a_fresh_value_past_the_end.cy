// trace: D133, D26
import std;
@config(default);

fn make [] -> std::array<i32>[len: 2 items]:
    return [4, 5];
end fn make

fn.main [] -> void:
    let $n: i32 = make()[pos: 3];
    return void;
end fn.main
