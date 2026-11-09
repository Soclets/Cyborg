// D143 says `pos:` doesn't say a unit, and nothing says this spelling exists.
// trace: D144, D143
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    let $n: i32 = $xs[pos.items: 1];
    return void;
end fn.main
