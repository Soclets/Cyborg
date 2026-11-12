// A list with no `len.max` is a view: `std::coll::list<i32>` reads any list or array, whole,
// and `std::coll::list.mut<i32>` changes its items, lent with `Lend.mut`.
// trace: D379
import std;
import os;

fn total [$xs: std::coll::list<i32>] -> i32:
    let mut $sum: i32 = 0;
    for $x in Lend $xs:
        change $sum = $sum + $x;
    end for
    return $sum + $xs[pos: 1] - $xs[pos: 1];
end fn total

fn double [$xs: std::coll::list.mut<i32>] -> void:
    for $i in range[from: 1, to: $xs.len]:
        change $xs[pos: $i] = $xs[pos: $i] x 2;
    end for
    return void;
end fn double

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $ys: std::coll::list<i32>[len.max: 8 items] = [1, 2, 3];
    let mut $zs: std::coll::array<i32>[len: 2 items] = [10, 20];
    double(Lend.mut $ys);
    double(Lend.mut $zs);
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", total(Lend $ys), total(Lend $zs), $ys.len);
    return void;
end fn.main
