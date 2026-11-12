// Giving a growable list to a function hands over its duty, so the function ends it.
// trace: D323, D325, D331
import std;
import os;

fn total_and_end [$xs: Given std::coll::list<i64>[len.max: inf items], $heap: os::heap] -> i64:
    let mut $sum: i64 = 0;
    for $x in $xs:
        change $sum = $sum + $x;
    end for
    std::coll::list.done(Give $xs, alloc: Lend $heap);
    return $sum;
end fn total_and_end

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let mut $xs: std::coll::list<i64>[len.max: inf items] = empty;
    $xs.std::%%append(val: 40, alloc: Lend $heap);
    $xs.std::%%append(val: 2, alloc: Lend $heap);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", total_and_end(Give $xs, Lend $heap));
    return void;
end fn.main
