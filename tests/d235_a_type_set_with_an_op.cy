// A type set lists types, not operations.
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

type_set odd:
    i32;
    op below [$a: i32, $b: i32] returns bool;
end type_set odd

fn.main [none] returns void:
    return void;
end fn.main
