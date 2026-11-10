import std;
use std::io::fmt_print as fmt_print;

fn fib [$n: i64] -> i64:
    if $n < 2:
        return $n;
    end if
    return fib($n - 1) + fib($n - 2);
end fn fib

fn.main [none] -> void:
    let $r: i64 = fib(35);
    fmt_print("{$r}{newln}");
    return void;
end fn.main
