// Which interfaces `std` declares for its own types, and their names, aren't decided.
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

fn larger<?item: std::ordered> [$a: ?item, $b: ?item] -> bool:
    return true;
end fn larger

fn.main [none] -> void:
    return void;
end fn.main
