// A range of numbers with a unit isn't decided, nor is `pos:` taking one.
// trace: D140, D128
import std;

fn.main [] -> void:
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    for $i in range[from: 1 items, to: $a.len]:
        return void;
    end for
    return void;
end fn.main
