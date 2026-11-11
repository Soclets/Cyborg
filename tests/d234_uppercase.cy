// A type parameter is `?` and a lowercase name.
// trace: D234
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

fn first<?T: any> [$a: Given ?T] -> ?T:
    return Give $a;
end fn first

fn.main [none] -> void:
    return void;
end fn.main
