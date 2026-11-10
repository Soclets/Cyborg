// A type set is closed, so nothing joins it with `satisfies`.
// trace: D235
@config(default);
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

u8 satisfies whole;

fn.main [] -> void:
    return void;
end fn.main
