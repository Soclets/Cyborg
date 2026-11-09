// trace: D144
import std;
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    let $n: i32 = $xs[offset.items: 1 time.seconds];
    return void;
end fn.main
