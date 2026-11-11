// A body uses on a `?item` only what its requirement lists.
// trace: D235, D239
import std;

interface ordered<?self>:
    op below [$a: ?self, $b: ?self] -> bool;
end interface ordered

interface addable<?self>:
    op + [$a: ?self, $b: ?self] -> ?self;
end interface addable

type_set whole:
    i32;
    i64;
end type_set whole

fn larger<?item: any.type> [$a: ?item, $b: ?item] -> bool:
    return $a below $b;
end fn larger

fn.main [none] -> void:
    return void;
end fn.main
