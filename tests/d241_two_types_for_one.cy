// A call fills each type parameter with one type.
// trace: D241
import std;

interface ordered<?self>:
    op below [$a: ?self, $b: ?self] returns bool;
end interface ordered

interface addable<?self>:
    op + [$a: ?self, $b: ?self] returns ?self;
end interface addable

type_set whole:
    i32;
    i64;
end type_set whole

i32 satisfies addable;
i64 satisfies addable;

fn total<?item: whole and addable> [$a: ?item, $b: ?item] returns ?item:
    return $a + $b;
end fn total

fn.main [none] returns void:
    let $a: i32 set_to 1;
    let $b: i64 set_to 2;
    let $c: i64 set_to total($a, $b);
    return void;
end fn.main
