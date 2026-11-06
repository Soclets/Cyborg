// Whether `pos:` takes a unit isn't decided.
// trace: D133, D101
import std;

fn.main [] -> void:
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    let $n: i32 = $a[pos: 1 items];
    return void;
end fn.main
