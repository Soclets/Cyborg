// When only plain numbers can fill `?item`, like a `type_set` of `i32` and `i64`, there's no
// choice, so the parameter is bare, and so are the calls.
// trace: D394, D235
import std;
import os;

interface addable<?self>:
    op + [$a: Lent ?self, $b: Lent ?self] -> ?self;
end interface addable

type_set whole:
    i32;
    i64;
end type_set whole

i32 satisfies addable;

fn total<?item: whole and addable> [$a: ?item, $b: ?item] -> ?item:
    return $a + $b;
end fn total

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 4;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", total($a, 5));
    return void;
end fn.main
