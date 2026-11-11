// trace: D144
import std;

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    let $n: i32 = $xs[offset.bytes: 1];
    return void;
end fn.main
