// trace: D140, D7
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 1 items] set_to [1];
    let $n: i32[unit: items] set_to 1 items;
    if $a.len == $n:
        return void;
    end if
    return void;
end fn.main
