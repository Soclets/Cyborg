// Lists built up and read back, with bounds and recursion standing in for what isn't decided yet.
// trace: D2, D4, D13, D20, D96, D115, D120
@config(default);
import std;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: 8 items] = empty;
    fill(Lend.mut $xs, 5);
    std::io::fmt_print("sum {}{newln}", sum(Lend $xs));
    std::io::fmt_print("largest {}{newln}", largest(Lend $xs));
    let mut $evens: std::list<i32>[len.max: 8 items] = empty;
    keep_even(Lend $xs, Lend.mut $evens);
    print_all(Lend $evens);
    return void;
end fn.main

fn fill [$xs: Lent.mut std::list<i32>[len.max: 8 items], $n: i32[min: 0, max: 8]] -> void:
    if $n == 0:
        return void;
    end if
    $xs.std::%append(val: $n x $n);
    fill(Lend.mut $xs, $n - 1);
    return void;
end fn fill

fn sum [$xs: std::list<i32>[len.max: 8 items]] -> i32:
    let mut $total: i32 = 0;
    for $x in $xs:
        change $total = $total + $x;
    end for
    return $total;
end fn sum

fn largest [$xs: std::list<i32>[len.max: 8 items]] -> i32:
    requires $xs.len at_least 1 items;

    let mut $best: i32 = -2147483648;
    for $x in $xs:
        if $x > $best:
            change $best = $x;
        end if
    end for
    return $best;
end fn largest

fn keep_even [$xs: std::list<i32>[len.max: 8 items], $out: Lent.mut std::list<i32>[len.max: 8 items]] -> void:
    for $x in $xs:
        if $x mod.trunc 2 == 0:
            $out.std::%append(val: $x);
        end if
    end for
    return void;
end fn keep_even

fn print_all [$xs: std::list<i32>[len.max: 8 items]] -> void:
    for $x in $xs:
        std::io::fmt_print("{}{newln}", $x);
    end for
    return void;
end fn print_all
