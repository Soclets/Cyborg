// `-` overflowing isn't decided, and an unsigned length makes that easy to hit.
// trace: D140, D42
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 1 items] set_to [1];
    let $n: u64[unit: items] set_to $a.len - 2 items;
    return void;
end fn.main
