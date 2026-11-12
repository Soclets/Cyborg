// A `.mut` view changes what it views, so it's lent with `Lend.mut`.
// trace: D379, D2
import std;

fn double [$xs: std::coll::list.mut<i32>] -> void:
    return void;
end fn double

fn.main [none] -> void:
    let mut $ys: std::coll::list<i32>[len.max: 8 items] = [1, 2, 3];
    double(Lend $ys);
    return void;
end fn.main
