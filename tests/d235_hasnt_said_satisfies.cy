// A type fits an interface only through a `satisfies` line.
// trace: D235
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

fn larger<?item: ordered> [$a: Lent ?item, $b: Lent ?item] -> bool:
    return $a above $b;
end fn larger

fn.main [none] -> void:
    let $a: i32 = 4;
    let $b: bool = larger($a, $a);
    return void;
end fn.main
