// Only plain numbers can fill `?item` here, so `Lent` has no choice to name.
// trace: D394
import std;

type_set whole:
    i32;
    i64;
end type_set whole

fn same<?item: whole> [$a: Lent ?item] -> ?item:
    return $a;
end fn same

fn.main [none] -> void:
    return void;
end fn.main
