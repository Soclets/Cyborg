// The owner's sketch, from design/sketch.md.
// trace: D2, D4, D11, D13, D115
@config(default);
import std;

fn total [$xs: std::list<i32>[len.max: inf items]] -> i32:
    let mut $sum: i32 = 0;
    for $x in $xs:
        change $sum = $sum + $x;
    end for
    return $sum;
end fn total

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::%%append(val: 4);
    $xs.std::%%append(val: 6);
    $xs.std::%%append(val: 5);

    let $t: i32 = total(Lend $xs);
    std::io::fmt_print("count: {}{newln}", $xs.len);
    if $t at_least 10:
        std::io::fmt_print("big total: {}{newln}", $t);
    end if
    return void;
end fn.main
