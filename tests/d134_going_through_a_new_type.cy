// trace: D134, D128
import std;

type.new row set_to std::array<i32>[len: 2 items];

fn f [$r: row] returns void:
    for $x in $r:
        return void;
    end for
    return void;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
