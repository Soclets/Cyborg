// How a type slot is written isn't decided.
// trace: D238
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

interface container<?self, ?item>:
    op + [$a: ?self, $b: ?self] returns ?self;
end interface container

fn.main [none] returns void:
    return void;
end fn.main
