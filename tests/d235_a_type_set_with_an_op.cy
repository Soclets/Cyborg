// A type set lists types, not operations.
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

type_set odd:
    i32;
    op < [$a: i32, $b: i32] -> bool;
end type_set odd

fn.main [none] -> void:
    return void;
end fn.main
