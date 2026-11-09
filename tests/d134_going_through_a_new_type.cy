// trace: D134, D128
@config(default);
import std;

type.new row = std::array<i32>[len: 2 items];

fn f [$r: row] -> void:
    for $x in $r:
        return void;
    end for
    return void;
end fn f

fn.main [] -> void:
    return void;
end fn.main
