// A body uses on a `?item` only what its requirement lists.
// trace: D235, D239
import std;

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

fn larger<?item: any.type> [$a: Lent ?item, $b: Lent ?item] -> bool:
    return $a below $b;
end fn larger

fn.main [none] -> void:
    return void;
end fn.main
