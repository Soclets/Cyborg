// trace: D133, D7
@config(default);
import std;

fn.main [] -> void:
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    let $s: str = $a[pos: 1];
    return void;
end fn.main
