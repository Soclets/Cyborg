// `satisfies` is checked where it's written, so the type needs every operation listed.
// trace: D235, D236
import std;

interface ordered<?self>:
    op below [$a: Lent ?self, $b: Lent ?self] -> bool;
end interface ordered

interface addable<?self>:
    op + [$a: Lent ?self, $b: Lent ?self] -> ?self;
end interface addable

type_set whole:
    i32;
    i64;
end type_set whole

struct reading:
    celsius: i32;
end struct reading

reading satisfies ordered;

fn.main [none] -> void:
    return void;
end fn.main
