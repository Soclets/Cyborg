// A body uses on a `?item` only what its requirement lists.
// trace: D235, D239
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

fn larger<?item: any.type> [$a: ?item, $b: ?item] returns bool:
    return $a below $b;
end fn larger

fn.main [none] returns void:
    return void;
end fn.main
