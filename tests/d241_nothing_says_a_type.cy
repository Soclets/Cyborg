// With bare literals and nowhere for the result to go, nothing says the type.
// trace: D241, D50
import std;
import os;

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

i32 satisfies addable;

fn total<?item: whole and addable> [$a: ?item, $b: ?item] returns ?item:
    return $a + $b;
end fn total

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", total(1, 2));
    return void;
end fn.main
