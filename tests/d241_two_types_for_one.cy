// A call fills each type parameter with one type.
// trace: D241
import std;

interface ordered<?self>:
    op < [$a: ?self, $b: ?self] -> bool;
end interface ordered

interface addable<?self>:
    op + [$a: ?self, $b: ?self] -> ?self;
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
    let $a: i32 = 1;
    let $b: i64 = 2;
    let $c: i64 = total($a, $b);
    return void;
end fn.main
