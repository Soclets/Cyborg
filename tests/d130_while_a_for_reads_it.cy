// trace: D130, D2
import std;

struct bag:
    items: std::coll::array<i32>[len: 1 items];
end struct bag

fn bag.%%empty_out [$b: Lent.mut bag] -> void:
    return void;
end fn bag.%%empty_out

fn.main [none] -> void:
    let mut $b: bag = bag[items = [1]];
    for $x in $b.items:
        $b.%%empty_out();
    end for
    return void;
end fn.main
