// The owner's sketch, from design/sketch.md.
// trace: D2, D4, D11, D13, D115
import std;
import os;

fn total [$xs: Lent std::coll::list<i32>[len.max: inf items]] -> i32:
    let mut $sum: i32 = 0;
    for $x in Lend $xs:
        change $sum = $sum + $x;
    end for
    return $sum;
end fn total

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = empty;
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    $xs.std::%%append(val: 6, alloc: Lend $heap);
    $xs.std::%%append(val: 5, alloc: Lend $heap);

    let $t: i32 = total(Lend $xs);
    std::io::fmt_print(Lend.mut $out, "count: {}{newln}", $xs.len);
    if $t at_least 10:
        std::io::fmt_print(Lend.mut $out, "big total: {}{newln}", $t);
    end if
    return void;
end fn.main
