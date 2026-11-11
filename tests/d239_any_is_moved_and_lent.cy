// An `any` parameter can be stored, moved, copied and lent, and nothing more.
// trace: D239
import std;
import os;

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

struct box<?item: any.type>:
    held: ?item;
end struct box

fn keep<?item: any.type> [$a: Given ?item] returns ?item:
    let $b: ?item set_to Give $a;
    return Give $b;
end fn keep

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $text: str set_to "kept";
    let $k: str set_to keep(Give $text);
    let $b: box<i32> set_to box[held set_to 5];
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $k, $b.held);
    return void;
end fn.main
