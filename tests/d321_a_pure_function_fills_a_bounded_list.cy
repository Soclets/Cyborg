// A list with a finite `len.max` holds its items inside it and takes no allocator, so a pure
// function can fill one.
// trace: D321, D86
import std;
import os;

fn.pure second [$a: i32, $b: i32] -> i32:
    let $xs: std::coll::list<i32>[len.max: 4 items] = [$a, $b];
    return $xs[pos: 2];
end fn.pure second

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", second(3, 4));
    return void;
end fn.main
