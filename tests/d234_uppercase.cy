// A type parameter is `?` and a lowercase name.
// trace: D234
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

fn first<?T: any> [$a: Given ?T] returns ?T:
    return Give $a;
end fn first

fn.main [none] returns void:
    return void;
end fn.main
