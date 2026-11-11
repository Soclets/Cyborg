// trace: D144
import std;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 3 items] set_to [4, 5, 6];
    let $n: i32 set_to $xs[offset.items: 1 time.seconds];
    return void;
end fn.main
