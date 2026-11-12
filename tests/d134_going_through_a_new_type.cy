// trace: D134, D128
import std;

type.new row = std::coll::array<i32>[len: 2 items];

fn f [$r: Lent row] -> void:
    for $x in Lend $r:
        return void;
    end for
    return void;
end fn f

fn.main [none] -> void:
    return void;
end fn.main
