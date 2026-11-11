// Two fields don't touch, but nothing says whether `for` is using the whole variable.
// trace: D125, D2
import std;

struct bag:
    items: std::array<i32>[len: 2 items];
    total: i32;
end struct bag

fn.main [none] returns void:
    let mut $b: bag set_to bag[items set_to [1, 2], total set_to 0];
    for $x in $b.items:
        change $b.total set_to $b.total + $x;
    end for
    return void;
end fn.main
