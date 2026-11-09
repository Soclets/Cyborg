// trace: D140, D7
import std;
@config(default);

fn.main [] -> void:
    let $a: std::array<i32>[len: 1 items] = [1];
    let $n: i32.items = 1 items;
    if $a.len == $n:
        return void;
    end if
    return void;
end fn.main
