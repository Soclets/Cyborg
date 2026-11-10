// An `any` parameter can be stored, moved, copied and lent, and nothing more.
// trace: D239
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

struct box<?item: any>:
    held: ?item;
end struct box

fn keep<?item: any> [$a: Given ?item] -> ?item:
    let $b: ?item = Give $a;
    return Give $b;
end fn keep

fn.main [] -> void:
    let $text: str = "kept";
    let $k: str = keep(Give $text);
    let $b: box<i32> = box[held = 5];
    std::io::fmt_print("{} {}{newln}", $k, $b.held);
    return void;
end fn.main
