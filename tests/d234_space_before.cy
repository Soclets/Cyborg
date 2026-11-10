// Type parameters go straight after the name.
// trace: D234
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

fn first <?item: any.type> [$a: Given ?item] -> ?item:
    return Give $a;
end fn first

fn.main [none] -> void:
    return void;
end fn.main
