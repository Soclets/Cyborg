// Type parameters go straight after the name.
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

fn first <?item: any.type> [$a: Given ?item] returns ?item:
    return Give $a;
end fn first

fn.main [none] returns void:
    return void;
end fn.main
