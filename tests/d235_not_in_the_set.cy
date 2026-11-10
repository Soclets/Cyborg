// A type set is a closed list of the types it allows.
// trace: D235
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

u8 satisfies addable;

fn total<?item: whole and addable> [$a: ?item, $b: ?item] -> ?item:
    return $a + $b;
end fn total

fn.main [none] -> void:
    let $a: u8 = 4;
    let $b: u8 = total($a, $a);
    return void;
end fn.main
