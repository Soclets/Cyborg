// trace: D134, D133
import std;
@config(default);

type.new row = std::array<i32>[len: 2 items];

fn f [$r: row] -> i32:
    return $r[pos: 1];
end fn f

fn.main [] -> void:
    return void;
end fn.main
