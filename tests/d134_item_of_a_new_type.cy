// trace: D134, D133
import std;

type.new row = std::array<i32>[len: 2 items];

fn f [$r: row] -> i32:
    return $r[pos: 1];
end fn f

fn.main [none] -> void:
    return void;
end fn.main
