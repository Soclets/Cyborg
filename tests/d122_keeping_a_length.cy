// A length's type isn't stated.
// trace: D122
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    let $n: i32 = $xs.len;
    return void;
end fn.main
