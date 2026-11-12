// trace: D134
import std;
import os;

type.alias score = i32;
type.alias numbers = std::coll::list<i32>[len.max: 8 items];

fn total [$xs: Lent numbers] -> score:
    let mut $sum: i32 = 0;
    for $x in Lend $xs:
        change $sum = $sum + $x;
    end for
    return $sum;
end fn total

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: score = 5;
    let $b: i32 = $a;
    let $c: score = $b + 1;
    let mut $xs: std::coll::list<i32>[len.max: 8 items] = empty;
    $xs.std::%%append(val: $c);
    let $ys: numbers = Copy $xs;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", total(Lend $ys));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", total(Lend $xs));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main
