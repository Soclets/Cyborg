// The owner's sketch, from design/sketch.md.
// trace: D2, D4, D11, D13, D115
import std;
import os;

fn total [$xs: std::list<i32>[len.max: inf items]] returns i32:
    let mut $sum: i32 set_to 0;
    for $x in $xs:
        change $sum set_to $sum + $x;
    end for
    return $sum;
end fn total

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: inf items] set_to empty;
    $xs.std::%%append(val: 4);
    $xs.std::%%append(val: 6);
    $xs.std::%%append(val: 5);

    let $t: i32 set_to total(Lend $xs);
    std::io::fmt_print(Lend.mut $out, "count: {}{newln}", $xs.len);
    if $t at_least 10:
        std::io::fmt_print(Lend.mut $out, "big total: {}{newln}", $t);
    end if
    return void;
end fn.main
