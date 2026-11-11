// A type fits an interface only through a `satisfies` line.
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

fn larger<?item: ordered> [$a: ?item, $b: ?item] returns bool:
    return $a above $b;
end fn larger

fn.main [none] returns void:
    let $a: i32 set_to 4;
    let $b: bool set_to larger($a, $a);
    return void;
end fn.main
