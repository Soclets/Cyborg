// trace: D133
import std;
@config(default);

fn.main [] -> void:
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    let $n: i32 = $a[pos: 1, offset: 0];
    return void;
end fn.main
