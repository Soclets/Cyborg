// trace: D133, D26
import std;

fn make [none] returns std::array<i32>[len: 2 items]:
    return [4, 5];
end fn make

fn.main [none] returns void:
    let $n: i32 set_to make()[pos: 3];
    return void;
end fn.main
