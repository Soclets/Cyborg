// trace: D144
import std;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] -> void:
    let $xs: std::coll::array<i32>[len: 3 items] = [4, 5, 6];
    let $n: i32 = $xs[offset.items: 1 time.seconds];
    return void;
end fn.main
