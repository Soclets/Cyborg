// An interface lists operations, and a type set types, and the two are never mixed.
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

interface mixed<?self>:
    i32;
end interface mixed

fn.main [none] -> void:
    return void;
end fn.main
