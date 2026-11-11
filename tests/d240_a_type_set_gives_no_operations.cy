// Only an interface gives operations, so a type set alone allows no `+`.
// trace: D240
import std;

interface ordered<?self>:
    op below [$a: ?self, $b: ?self] -> bool;
end interface ordered

interface addable<?self>:
    op + [$a: ?self, $b: ?self] -> ?self;
end interface addable

type_set whole:
    i32;
    i64;
end type_set whole

fn total<?item: whole> [$a: ?item, $b: ?item] -> ?item:
    return $a + $b;
end fn total

fn.main [none] -> void:
    return void;
end fn.main
