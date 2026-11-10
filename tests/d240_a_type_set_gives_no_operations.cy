// Only an interface gives operations, so a type set alone allows no `+`.
// trace: D240
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

fn total<?item: whole> [$a: ?item, $b: ?item] -> ?item:
    return $a + $b;
end fn total

fn.main [] -> void:
    return void;
end fn.main
