// trace: D134
import std;
import os;

type.alias score set_to i32;
type.alias numbers set_to std::list<i32>[len.max: inf items];

fn total [$xs: numbers] returns score:
    let mut $sum: i32 set_to 0;
    for $x in $xs:
        change $sum set_to $sum + $x;
    end for
    return $sum;
end fn total

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: score set_to 5;
    let $b: i32 set_to $a;
    let $c: score set_to $b + 1;
    let mut $xs: std::list<i32>[len.max: inf items] set_to empty;
    $xs.std::%%append(val: $c);
    let $ys: numbers set_to Copy $xs;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", total(Lend $ys));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", total(Lend $xs));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main
