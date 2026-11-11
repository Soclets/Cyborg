// trace: D134, D133
import std;

type.new row set_to std::array<i32>[len: 2 items];

fn f [$r: row] returns i32:
    return $r[pos: 1];
end fn f

fn.main [none] returns void:
    return void;
end fn.main
