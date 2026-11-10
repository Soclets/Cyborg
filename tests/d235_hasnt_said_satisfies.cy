// A type fits an interface only through a `satisfies` line.
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

fn larger<?item: ordered> [$a: ?item, $b: ?item] -> bool:
    return $a > $b;
end fn larger

fn.main [] -> void:
    let $a: i32 = 4;
    let $b: bool = larger($a, $a);
    return void;
end fn.main
