// A generic can call another with its own `?item`, lent like any other value, and it needs what the other needs.
// trace: D235, D241
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

i32 satisfies ordered;

fn larger<?item: ordered> [$a: ?item, $b: ?item] returns bool:
    return $a above $b;
end fn larger

fn smaller<?thing: ordered> [$a: ?thing, $b: ?thing] returns bool:
    return larger(Lend $b, Lend $a);
end fn smaller

fn shown [$yes: bool] returns str:
    if $yes:
        return "yes";
    end if
    return "no";
end fn shown

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32 set_to 3;
    let $b: i32 set_to 8;
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", shown(smaller($a, $b)), shown(smaller($b, $a)));
    return void;
end fn.main
