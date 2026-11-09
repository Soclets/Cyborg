// `-` overflowing isn't decided, and an unsigned length makes that easy to hit.
// trace: D140, D42
@config(default);
import std;

fn.main [] -> void:
    let $a: std::array<i32>[len: 1 items] = [1];
    let $n: u64.items = $a.len - 2 items;
    return void;
end fn.main
