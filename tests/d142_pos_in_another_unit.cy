// trace: D142, D144
import std;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $xs: std::array<i32>[len: 1 items] = [1];
    let $n: i32 = $xs[pos: 1 time.seconds];
    return void;
end fn.main
