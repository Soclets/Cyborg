// An interface lists operations, and a type set types, and the two are never mixed.
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

interface mixed<?self>:
    i32;
end interface mixed

fn.main [none] returns void:
    return void;
end fn.main
