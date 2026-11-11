// trace: D142, D144
import std;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 1 items] set_to [1];
    let $n: i32 set_to $xs[pos: 1 time.seconds];
    return void;
end fn.main
