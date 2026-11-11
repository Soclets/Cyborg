// trace: D133, D20
import std;

fn.main [none] -> void:
    let $a: std::array<i32>[len: 1 items] = [50];
    let $b: i32[min: 0, max: 9] = $a[pos: 1];
    return void;
end fn.main
