// How a type slot is written isn't decided.
// trace: D238
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

interface container<?self, ?item>:
    op + [$a: ?self, $b: ?self] -> ?self;
end interface container

fn.main [] -> void:
    return void;
end fn.main
