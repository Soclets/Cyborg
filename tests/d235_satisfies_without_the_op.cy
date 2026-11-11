// `satisfies` is checked where it's written, so the type needs every operation listed.
// trace: D235, D236
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

struct reading:
    celsius: i32;
end struct reading

reading satisfies ordered;

fn.main [none] returns void:
    return void;
end fn.main
