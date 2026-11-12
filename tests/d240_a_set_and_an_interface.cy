// A type set and an interface together: the set says which types, the interface what they do.
// trace: D240, D235
import std;
import os;

interface ordered<?self>:
    op below [$a: Lent ?self, $b: Lent ?self] -> bool;
end interface ordered

interface addable<?self>:
    op + [$a: Lent ?self, $b: Lent ?self] -> ?self;
end interface addable

type_set whole:
    i32;
    i64;
end type_set whole

i32 satisfies addable;
i64 satisfies addable;

fn total<?item: whole and addable> [$a: ?item, $b: ?item] -> ?item:
    return $a + $b;
end fn total

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i64 = 40;
    let $b: i64 = 2;
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", total($a, $b), total($b, $a));
    let $small: i32 = total(1, 2);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $small);
    return void;
end fn.main
