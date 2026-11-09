// `pos:` counts from 1, so 0 is a bad position.
// trace: D133, D26
import std;
@config(default);

fn.main [] -> void:
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    let $n: i32 = $a[pos: 0];
    return void;
end fn.main
