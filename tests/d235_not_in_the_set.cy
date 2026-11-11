// A type set is a closed list of the types it allows.
// trace: D235
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

u8 satisfies addable;

fn total<?item: whole and addable> [$a: ?item, $b: ?item] returns ?item:
    return $a + $b;
end fn total

fn.main [none] returns void:
    let $a: u8 set_to 4;
    let $b: u8 set_to total($a, $a);
    return void;
end fn.main
