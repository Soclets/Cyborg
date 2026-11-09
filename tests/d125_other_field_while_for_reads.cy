// Two fields don't touch, but nothing says whether `for` is using the whole variable.
// trace: D125, D2
import std;
@config(default);

struct bag:
    items: std::array<i32>[len: 2 items];
    total: i32;
end struct bag

fn.main [] -> void:
    let mut $b: bag = bag[items = [1, 2], total = 0];
    for $x in $b.items:
        change $b.total = $b.total + $x;
    end for
    return void;
end fn.main
