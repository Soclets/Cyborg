// An `any` parameter can be stored, moved, copied and lent, and nothing more.
// trace: D239
import std;
import os;

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

struct box<?item: any.type>:
    held: ?item;
end struct box

fn keep<?item: any.type> [$a: Given ?item] -> ?item:
    let $b: ?item = Give $a;
    return Give $b;
end fn keep

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $text: str[len.max: 32 bytes] = "kept";
    let $k: str[len.max: 32 bytes] = keep(Give $text);
    let $b: box<i32> = box[held = 5];
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $k, $b.held);
    return void;
end fn.main
