// `$xs.%%split.mut(at.items: 2)` gives a `std::coll::halves.mut<i64>`, whose `.first` and
// `.second` are views of two parts that don't overlap, so two tasks can change them at once.
// Changes go into the list itself. `$xs.%split` gives two parts to read.
// trace: D382, D358, D347, D379
import std;
import os;
import @concur;

fn double [$xs: std::coll::list.mut<i64>] -> i64:
    for $i in range[from: 1, to: $xs.len]:
        change $xs[pos: $i] = $xs[pos: $i] x 2;
    end for
    return 1;
end fn double

fn total [$xs: std::coll::list<i64>] -> i64:
    let mut $sum: i64 = 0;
    for $x in $xs:
        change $sum = $sum + $x;
    end for
    return $sum;
end fn total

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $pool: os::threads = os::threads(count: 2);
    let mut $xs: std::coll::list<i64>[len.max: 8 items] = [1, 2, 3, 4, 5];
    let $h: std::coll::halves.mut<i64> = $xs.%%split.mut(at.items: 2);
    let $a: @concur::task<i64> = @concur::spawn(on: Lend $pool) double(Lend.mut $h.first);
    let $b: @concur::task<i64> = @concur::spawn(on: Lend $pool) double(Lend.mut $h.second);
    let $counted: i64 = @concur::task.done(Give $a) + @concur::task.done(Give $b);
    let $r: std::coll::halves<i64> = $xs.%split(at.items: 4);
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $counted, total($r.first), total($r.second), $xs[pos: 5]);
    return void;
end fn.main
