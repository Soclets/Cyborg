// trace: D134
@config(default);
import std;

type.alias score = i32;
type.alias numbers = std::list<i32>[len.max: inf items];

fn total [$xs: numbers] -> score:
    let mut $sum: i32 = 0;
    for $x in $xs:
        change $sum = $sum + $x;
    end for
    return $sum;
end fn total

fn.main [] -> void:
    let $a: score = 5;
    let $b: i32 = $a;
    let $c: score = $b + 1;
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::%append(val: $c);
    let $ys: numbers = Copy $xs;
    std::io::fmt_print("{}{newln}", total(Lend $ys));
    std::io::fmt_print("{}{newln}", total(Lend $xs));
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
