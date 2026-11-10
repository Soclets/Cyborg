// With bare literals and nowhere for the result to go, nothing says the type.
// trace: D241, D50
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

fn total<?item: whole and addable> [$a: ?item, $b: ?item] -> ?item:
    return $a + $b;
end fn total

fn.main [none] -> void:
    let $x := total(1, 2);
    return void;
end fn.main
